use std::env;

use ree_pak_core::filename::FileNameTable;

use crate::{
    channel::ProgressChannel,
    error::{Error, Result},
    filename::FileListInfo,
    pak::{
        ExtractOptions, Pak, PakId, PakInfo,
        group::PakGroup,
        tree::{FileTree, RenderTreeNode, RenderTreeOptions},
    },
};

pub fn pak_clear_all() {
    PakGroup::instance().lock().unwrap().remove_all_paks();
}

pub fn pak_list_all() -> Vec<PakInfo> {
    PakGroup::instance().lock().unwrap().pak_infos()
}

pub fn pak_open(path: &str) -> Result<PakId> {
    // open pak file
    let file = std::fs::File::open(path).map_err(|e| Error::FileIO {
        path: path.to_string(),
        source: e,
    })?;
    let mut reader = std::io::BufReader::new(file);
    let archive = ree_pak_core::read::read_archive(&mut reader)?;

    // store pak and create id
    let path_abs = std::path::Path::new(path).canonicalize()?;
    let pak = Pak::new(&path_abs.display().to_string(), archive, reader);
    let id: PakId = pak.id();

    PakGroup::instance().lock().unwrap().add_pak(pak);
    Ok(id)
}

pub fn pak_close(id: PakId) -> Result<()> {
    if PakGroup::instance().lock().unwrap().remove_pak(&id).is_none() {
        return Err(Error::PakIdNotFound(id));
    };
    Ok(())
}

pub fn pak_order(order: &[PakId]) -> Result<()> {
    let pak_group = PakGroup::instance();
    let mut pak_group = pak_group.lock().unwrap();
    let paks = pak_group.paks_mut();
    // check if order list is valid
    if order.len() != paks.len() {
        return Err(Error::InvalidOrder(
            "Order list length does not match number of paks.".to_string(),
        ));
    }
    let all_found = order.iter().all(|id| paks.iter().any(|pak| pak.id() == *id));
    if !all_found {
        return Err(Error::InvalidOrder("Order list contains unknown pak ids.".to_string()));
    }
    // sort paks by order list
    paks.sort_by_key(|pak| order.iter().position(|id| pak.id() == *id).unwrap());
    Ok(())
}

pub fn pak_get_info(id: PakId) -> Result<PakInfo> {
    if let Some(pak) = PakGroup::instance().lock().unwrap().get_pak(&id) {
        Ok(PakInfo {
            id,
            path: pak.path().to_string(),
        })
    } else {
        Err(Error::PakIdNotFound(id))
    }
}

pub fn pak_read_file_tree() -> Result<FileTree> {
    PakGroup::instance().lock().unwrap().render_tree_combined()
}

pub fn pak_read_file_tree_optimized(options: &RenderTreeOptions) -> Result<RenderTreeNode> {
    let basic_tree = PakGroup::instance().lock().unwrap().render_tree_combined()?;
    RenderTreeNode::try_from_file_tree(basic_tree, options)
}

pub fn pak_extract_all(options: &ExtractOptions, progress: ProgressChannel) -> Result<()> {
    PakGroup::instance().lock().unwrap().unpack_optional(options, progress)
}

pub fn get_file_lists() -> Result<Vec<FileListInfo>> {
    // try work path
    let mut work_path = env::current_dir().unwrap_or(".".into());
    work_path.push("assets");
    work_path.push("filelist");
    let result = FileListInfo::walk_dir(&work_path);
    if let Ok(result) = result {
        if !result.is_empty() {
            return Ok(result);
        }
    }

    // try exe path
    let mut exe_path = env::current_exe().unwrap_or(".".into());
    exe_path.push("assets");
    exe_path.push("filelist");
    let result = FileListInfo::walk_dir(&exe_path);
    match result {
        Ok(result) => Ok(result),
        Err(e) => Err(Error::FileListNotFound(format!(
            "error: {}, path: {} + {}",
            e,
            work_path.display(),
            exe_path.display()
        ))),
    }
}

pub fn load_file_list(path: &str) -> Result<()> {
    let table = FileNameTable::from_list_file(path)?;
    PakGroup::instance().lock().unwrap().set_file_name_table(table);
    Ok(())
}
