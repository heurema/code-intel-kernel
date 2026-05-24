#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelDatabase {
    pub path: String,
    pub read_only: bool,
    pub status: &'static str,
}

pub fn open_kernel_database(path: impl Into<String>) -> KernelDatabase {
    KernelDatabase {
        path: path.into(),
        read_only: true,
        status: "placeholder",
    }
}
