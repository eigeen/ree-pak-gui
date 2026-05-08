use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use rayon::prelude::*;

use crate::{
    channel::AudioExportProgressChannel,
    error::{Error, Result},
    external_tools,
};

use super::{AudioExtractBatchOptions, AudioService};

const MAX_AUDIO_CONVERSION_THREADS: usize = 8;

pub(super) trait AudioExportProgressSink: Clone {
    fn work_start(&self, count: u32);
    fn file_done(&self, path: &str);
    fn work_finished(&self);
}

impl AudioExportProgressSink for AudioExportProgressChannel {
    fn work_start(&self, count: u32) {
        AudioExportProgressChannel::work_start(self, count);
    }

    fn file_done(&self, path: &str) {
        AudioExportProgressChannel::file_done(self, path);
    }

    fn work_finished(&self) {
        AudioExportProgressChannel::work_finished(self);
    }
}

pub(super) fn extract_wavs(
    service: &AudioService,
    options: AudioExtractBatchOptions,
) -> Result<Vec<PathBuf>> {
    let output_dir = service.resolve_output_dir(options.output_dir);
    let wems = service.extract_wems_to_dir(options.source, &options.indices, &service.temp_dir)?;

    convert_wem_batch(WavConversionJob {
        wems,
        output_dir,
        should_terminate: None,
        progress: None::<AudioExportProgressChannel>,
        convert: convert_wem_to_wav,
    })
}

pub(super) fn extract_wavs_with_progress(
    service: &AudioService,
    options: AudioExtractBatchOptions,
    progress: AudioExportProgressChannel,
) -> Result<Vec<PathBuf>> {
    let output_dir = service.resolve_output_dir(options.output_dir);
    let should_terminate = service.export_should_terminate();

    progress.work_start(options.indices.len() as u32);
    stop_if_terminated(&should_terminate)?;
    let wems = service.extract_wems_to_dir(options.source, &options.indices, &service.temp_dir)?;
    let paths = convert_wem_batch(WavConversionJob {
        wems,
        output_dir,
        should_terminate: Some(should_terminate),
        progress: Some(progress.clone()),
        convert: convert_wem_to_wav,
    })?;
    progress.work_finished();

    Ok(paths)
}

struct WavConversionJob<P, F> {
    wems: Vec<PathBuf>,
    output_dir: PathBuf,
    should_terminate: Option<Arc<AtomicBool>>,
    progress: Option<P>,
    convert: F,
}

fn convert_wem_batch<P, F>(job: WavConversionJob<P, F>) -> Result<Vec<PathBuf>>
where
    P: AudioExportProgressSink + Send + Sync,
    F: Fn(&Path, &Path) -> Result<()> + Send + Sync,
{
    let thread_count = conversion_thread_count(job.wems.len());
    std::fs::create_dir_all(&job.output_dir)?;

    let pool = build_conversion_thread_pool(thread_count)?;
    pool.install(|| convert_wems_in_parallel(&job))
}

fn conversion_thread_count(file_count: usize) -> usize {
    let available = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    file_count
        .min(available)
        .min(MAX_AUDIO_CONVERSION_THREADS)
        .max(1)
}

fn build_conversion_thread_pool(thread_count: usize) -> Result<rayon::ThreadPool> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .thread_name(|index| format!("audio-export-{index}"))
        .build()
        .map_err(|error| Error::Internal(format!("Failed to build audio export pool: {error}")))
}

fn convert_wems_in_parallel<P, F>(job: &WavConversionJob<P, F>) -> Result<Vec<PathBuf>>
where
    P: AudioExportProgressSink + Send + Sync,
    F: Fn(&Path, &Path) -> Result<()> + Send + Sync,
{
    job.wems
        .par_iter()
        .enumerate()
        .map(|(index, wem_path)| convert_wem_entry(job, index, wem_path))
        .collect()
}

fn convert_wem_entry<P, F>(
    job: &WavConversionJob<P, F>,
    _index: usize,
    wem_path: &Path,
) -> Result<PathBuf>
where
    P: AudioExportProgressSink,
    F: Fn(&Path, &Path) -> Result<()>,
{
    stop_if_needed(&job.should_terminate)?;
    let wav_path = build_wav_path(wem_path, &job.output_dir)?;
    (job.convert)(wem_path, &wav_path)?;
    report_file_done(&job.progress, &wav_path);
    Ok(wav_path)
}

fn stop_if_needed(should_terminate: &Option<Arc<AtomicBool>>) -> Result<()> {
    match should_terminate {
        Some(flag) => stop_if_terminated(flag),
        None => Ok(()),
    }
}

fn stop_if_terminated(should_terminate: &AtomicBool) -> Result<()> {
    if should_terminate.load(Ordering::Relaxed) {
        return Err(Error::Terminated);
    }

    Ok(())
}

fn report_file_done<P>(progress: &Option<P>, wav_path: &Path)
where
    P: AudioExportProgressSink,
{
    if let Some(progress) = progress {
        progress.file_done(wav_path.to_string_lossy().as_ref());
    }
}

fn build_wav_path(wem_path: &Path, output_dir: &Path) -> Result<PathBuf> {
    let file_name = wem_path
        .with_extension("wav")
        .file_name()
        .ok_or_else(|| Error::Internal(format!("Invalid wem output path: {}", wem_path.display())))?
        .to_owned();

    Ok(output_dir.join(Path::new(&file_name)))
}

fn convert_wem_to_wav(wem_path: &Path, wav_path: &Path) -> Result<()> {
    if wav_path.exists() {
        return Ok(());
    }

    let cli_path = find_vgmstream_cli()?;
    let output = build_vgmstream_command(&cli_path, wem_path, wav_path)?.output()?;
    if output.status.success() {
        return Ok(());
    }

    Err(Error::VgmstreamCliFailed(command_error_detail(&output)))
}

fn find_vgmstream_cli() -> Result<PathBuf> {
    external_tools::find_vgmstream_cli().ok_or_else(|| {
        let expected = external_tools::vgmstream_cli_candidates()
            .into_iter()
            .next()
            .unwrap_or_else(|| external_tools::extension_dir("vgmstream"))
            .to_string_lossy()
            .to_string();
        Error::VgmstreamCliNotFound(expected)
    })
}

fn build_vgmstream_command(cli_path: &Path, wem_path: &Path, wav_path: &Path) -> Result<Command> {
    ensure_parent_dir(wav_path)?;
    let mut command = Command::new(cli_path);

    if let Some(parent) = cli_path.parent() {
        command.current_dir(parent);
        command.env(
            "DYLD_LIBRARY_PATH",
            build_child_library_path(parent, std::env::var_os("DYLD_LIBRARY_PATH")),
        );
    }

    command.args(["-i", "-o"]).arg(wav_path).arg(wem_path);
    Ok(command)
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    Ok(())
}

fn command_error_detail(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return stdout;
    }

    format!("exit status {}", output.status)
}

fn build_child_library_path(
    tool_dir: &Path,
    existing: Option<std::ffi::OsString>,
) -> std::ffi::OsString {
    let mut paths = vec![tool_dir.join("lib"), tool_dir.to_path_buf()];
    if let Some(existing) = existing {
        paths.extend(std::env::split_paths(&existing));
    }

    std::env::join_paths(paths).unwrap_or_else(|_| tool_dir.as_os_str().to_os_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Clone, Default)]
    struct RecordingProgress {
        events: Arc<Mutex<Vec<String>>>,
    }

    impl AudioExportProgressSink for RecordingProgress {
        fn work_start(&self, count: u32) {
            self.events.lock().unwrap().push(format!("start:{count}"));
        }

        fn file_done(&self, path: &str) {
            let mut events = self.events.lock().unwrap();
            let next = events
                .iter()
                .filter(|event| event.starts_with("file:"))
                .count()
                + 1;
            events.push(format!("file:{next}:{}", path_file_name(path)));
        }

        fn work_finished(&self) {
            self.events.lock().unwrap().push("finished".to_string());
        }
    }

    #[test]
    fn wav_conversion_returns_paths_in_order() {
        let temp_dir = tempfile::tempdir().unwrap();
        let progress = RecordingProgress::default();
        progress.work_start(2);

        let paths = convert_wem_batch(WavConversionJob {
            wems: test_wems(temp_dir.path(), &["b.wem", "a.wem"]),
            output_dir: temp_dir.path().join("out"),
            should_terminate: None,
            progress: Some(progress.clone()),
            convert: fake_convert,
        })
        .unwrap();
        progress.work_finished();

        assert_eq!(path_file_name(paths[0].to_string_lossy().as_ref()), "b.wav");
        assert_eq!(path_file_name(paths[1].to_string_lossy().as_ref()), "a.wav");
        assert_progress_reported_all_files(&progress, &["a.wav", "b.wav"]);
    }

    #[test]
    fn wav_conversion_stops_when_terminated() {
        let temp_dir = tempfile::tempdir().unwrap();
        let should_terminate = Arc::new(AtomicBool::new(true));

        let error = convert_wem_batch(WavConversionJob {
            wems: test_wems(temp_dir.path(), &["first.wem"]),
            output_dir: temp_dir.path().join("out"),
            should_terminate: Some(should_terminate),
            progress: Some(RecordingProgress::default()),
            convert: fake_convert,
        })
        .unwrap_err();

        assert!(matches!(error, Error::Terminated));
    }

    fn test_wems(dir: &Path, names: &[&str]) -> Vec<PathBuf> {
        names
            .iter()
            .map(|name| {
                let path = dir.join(name);
                std::fs::write(&path, b"wem").unwrap();
                path
            })
            .collect()
    }

    fn fake_convert(_wem_path: &Path, wav_path: &Path) -> Result<()> {
        std::fs::write(wav_path, b"wav")?;
        Ok(())
    }

    fn assert_progress_reported_all_files(progress: &RecordingProgress, expected: &[&str]) {
        let events = progress.events.lock().unwrap();
        assert_eq!(events.first().map(String::as_str), Some("start:2"));
        assert_eq!(events.last().map(String::as_str), Some("finished"));

        let mut file_names = events
            .iter()
            .filter_map(|event| event.split(':').nth(2))
            .collect::<Vec<_>>();
        file_names.sort_unstable();
        assert_eq!(file_names, expected);
    }

    fn path_file_name(path: &str) -> &str {
        Path::new(path)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(path)
    }
}
