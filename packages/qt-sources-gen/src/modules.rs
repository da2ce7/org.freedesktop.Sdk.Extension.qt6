pub const BASE_URL: &str = "https://download.qt.io/official_releases/qt";

pub const MODULES: &[&str] = &[
    "qtbase",
    "qtshadertools",
    "qtdeclarative",
    "qttools",
    "qtsvg",
    "qtimageformats",
    "qtwayland",
    "qtmultimedia",
    "qt5compat",
    "qtwebsockets",
    "qtwebchannel",
    "qtpositioning",
    "qtlocation",
    "qtconnectivity",
    "qtserialport",
    "qtsensors",
    "qtserialbus",
    "qtcharts",
    "qtnetworkauth",
    "qthttpserver",
    "qtlanguageserver",
    "qtscxml",
    "qtremoteobjects",
    "qtquicktimeline",
    "qtquick3d",
    "qtquick3dphysics",
    "qtspeech",
    "qtvirtualkeyboard",
    "qttranslations",
    "qtgrpc",
    "qtlottie",
    "qt3d",
    "qtdatavis3d",
    "qtgraphs",
    "qtopenapi",
    "qtcanvaspainter",
    "qttasktree",
];

/// Split "X.Y.Z" into ("X.Y", "Z"). Returns `None` if the format is invalid.
#[must_use]
pub fn split_version(version: &str) -> Option<(&str, &str)> {
    version.rsplit_once('.')
}

#[must_use]
pub fn archive_filename(module: &str, version: &str) -> String {
    format!("{module}-everywhere-src-{version}.tar.xz")
}

#[must_use]
pub fn archive_url(major_minor: &str, version: &str, filename: &str) -> String {
    format!("{BASE_URL}/{major_minor}/{version}/submodules/{filename}")
}

#[must_use]
pub fn md5sums_url(major_minor: &str, version: &str) -> String {
    format!("{BASE_URL}/{major_minor}/{version}/submodules/md5sums.txt")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_version_valid() {
        assert_eq!(split_version("6.11.0"), Some(("6.11", "0")));
    }

    #[test]
    fn split_version_no_dot() {
        assert_eq!(split_version("611"), None);
    }

    #[test]
    fn archive_filename_format() {
        assert_eq!(archive_filename("qtbase", "6.11.0"), "qtbase-everywhere-src-6.11.0.tar.xz");
    }

    #[test]
    fn archive_url_format() {
        let url = archive_url("6.11", "6.11.0", "qtbase-everywhere-src-6.11.0.tar.xz");
        assert_eq!(
            url,
            "https://download.qt.io/official_releases/qt/6.11/6.11.0/submodules/qtbase-everywhere-src-6.11.0.tar.xz"
        );
    }

    #[test]
    fn md5sums_url_format() {
        assert_eq!(
            md5sums_url("6.11", "6.11.0"),
            "https://download.qt.io/official_releases/qt/6.11/6.11.0/submodules/md5sums.txt"
        );
    }
}
