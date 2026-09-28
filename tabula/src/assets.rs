//! The page itself: HTML, CSS, JavaScript, and the Tabula mark, embedded in
//! the executable at build time so Tabula is a single file with no runtime
//! downloads.

/// An embedded file.
#[derive(Clone, Copy, Debug)]
pub struct Asset {
    /// The URL path it is served at.
    pub path: &'static str,
    /// Its media type.
    pub media_type: &'static str,
    /// Its bytes.
    pub bytes: &'static [u8],
}

/// Every embedded file.
pub const ASSETS: [Asset; 10] = [
    Asset {
        path: "/",
        media_type: "text/html; charset=utf-8",
        bytes: include_bytes!("../web/index.html"),
    },
    Asset {
        path: "/assets/tabula.css",
        media_type: "text/css; charset=utf-8",
        bytes: include_bytes!("../web/tabula.css"),
    },
    Asset {
        path: "/assets/tabula.svg",
        media_type: "image/svg+xml",
        bytes: include_bytes!("../web/tabula.svg"),
    },
    Asset {
        path: "/assets/app.js",
        media_type: "text/javascript; charset=utf-8",
        bytes: include_bytes!("../web/app.js"),
    },
    Asset {
        path: "/assets/api.js",
        media_type: "text/javascript; charset=utf-8",
        bytes: include_bytes!("../web/api.js"),
    },
    Asset {
        path: "/assets/orange.js",
        media_type: "text/javascript; charset=utf-8",
        bytes: include_bytes!("../web/orange.js"),
    },
    Asset {
        path: "/assets/editor.js",
        media_type: "text/javascript; charset=utf-8",
        bytes: include_bytes!("../web/editor.js"),
    },
    Asset {
        path: "/assets/markdown.js",
        media_type: "text/javascript; charset=utf-8",
        bytes: include_bytes!("../web/markdown.js"),
    },
    Asset {
        path: "/assets/ui.js",
        media_type: "text/javascript; charset=utf-8",
        bytes: include_bytes!("../web/ui.js"),
    },
    Asset {
        path: "/assets/values.js",
        media_type: "text/javascript; charset=utf-8",
        bytes: include_bytes!("../web/values.js"),
    },
];

/// Finds the embedded file served at `path`. `/index.html` is an alias of `/`.
#[must_use]
pub fn lookup(path: &str) -> Option<&'static Asset> {
    let path = if path == "/index.html" { "/" } else { path };
    ASSETS.iter().find(|asset| asset.path == path)
}

#[cfg(test)]
mod tests {
    use super::{ASSETS, lookup};

    #[test]
    fn every_asset_is_present_and_unique() {
        for asset in &ASSETS {
            assert!(!asset.bytes.is_empty(), "{} is empty", asset.path);
            assert_eq!(
                ASSETS
                    .iter()
                    .filter(|other| other.path == asset.path)
                    .count(),
                1
            );
        }
        assert!(lookup("/index.html").is_some());
        assert!(lookup("/assets/../Cargo.toml").is_none());
    }

    #[test]
    fn the_page_loads_only_embedded_scripts() {
        let page = std::str::from_utf8(lookup("/").unwrap().bytes).unwrap();
        assert!(page.contains(r#"<script type="module" src="/assets/app.js"></script>"#));
        assert!(!page.contains("http://") && !page.contains("https://"));
        assert!(!page.contains("style=\""));
        for asset in &ASSETS {
            if asset.path.ends_with(".js") {
                let script = std::str::from_utf8(asset.bytes).unwrap();
                assert!(!script.contains("eval("), "{} calls eval", asset.path);
                assert!(
                    !script.contains("new Function"),
                    "{} builds code",
                    asset.path
                );
            }
        }
    }
}
