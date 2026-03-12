/// Operating systems that polyws knows how to provision.
#[derive(Debug, Clone, PartialEq)]
pub enum OsType {
    Ubuntu,
    Debian,
    Arch,
    MacOs,
    Unknown(String),
}

/// Detect the OS type from the combined output of `uname -s` and
/// `/etc/os-release` (or similar) obtained via SSH.
pub fn detect(output: &str) -> OsType {
    let lower = output.to_lowercase();
    if lower.contains("ubuntu") {
        OsType::Ubuntu
    } else if lower.contains("debian") {
        OsType::Debian
    } else if lower.contains("arch") {
        OsType::Arch
    } else if lower.contains("darwin") || lower.contains("macos") {
        OsType::MacOs
    } else {
        OsType::Unknown(
            output
                .lines()
                .next()
                .unwrap_or("unknown")
                .trim()
                .to_string(),
        )
    }
}

impl std::fmt::Display for OsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OsType::Ubuntu => write!(f, "Ubuntu"),
            OsType::Debian => write!(f, "Debian"),
            OsType::Arch => write!(f, "Arch Linux"),
            OsType::MacOs => write!(f, "macOS"),
            OsType::Unknown(s) => write!(f, "Unknown ({})", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_ubuntu() {
        assert_eq!(detect("Linux\nNAME=\"Ubuntu 22.04\""), OsType::Ubuntu);
    }

    #[test]
    fn detects_macos() {
        assert_eq!(detect("Darwin"), OsType::MacOs);
    }

    #[test]
    fn detects_arch() {
        assert_eq!(detect("Linux\nNAME=\"Arch Linux\""), OsType::Arch);
    }
}
