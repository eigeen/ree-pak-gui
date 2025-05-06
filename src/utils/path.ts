export function parentPath(path: string): string {
    // '/' or '\'
    const lastSlashIndex = Math.max(
        path.lastIndexOf('/'),
        path.lastIndexOf('\\')
    );
    return path.substring(0, lastSlashIndex);
}