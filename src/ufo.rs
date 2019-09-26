//! Reading and (maybe) writing Unified Font Object files.

use crate::layer::Layer;
use std::path::{Path, PathBuf};

use crate::Error;

static LAYER_CONTENTS_FILE: &str = "layercontents.plist";

/// A Unified Font Object.
pub struct Ufo {
    layers: Vec<LayerInfo>,
}

/// A [font layer], along with its name and path.
///
/// This corresponds to a 'glyphs' directory on disk.
///
/// [font layer]: http://unifiedfontobject.org/versions/ufo3/glyphs/
pub struct LayerInfo {
    pub name: String,
    pub path: PathBuf,
    pub layer: Layer,
}

impl Ufo {
    /// Attempt to load a font object from a file. `path` must point to
    /// a directory with the structure described in [v3 of the Unified Font Object][v3]
    /// spec.
    ///
    /// [v3]: http://unifiedfontobject.org/versions/ufo3/
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Ufo, Error> {
        let path = path.as_ref();
        return load_impl(path);

        // minimize monomorphization
        fn load_impl(path: &Path) -> Result<Ufo, Error> {
            let contents_path = path.join(LAYER_CONTENTS_FILE);
            let mut contents: Vec<(String, PathBuf)> = plist::from_file(contents_path)?;
            let layers: Result<Vec<LayerInfo>, Error> = contents
                .drain(..)
                .map(|(name, p)| {
                    let layer_path = path.join(&p);
                    let layer = Layer::load(layer_path)?;
                    Ok(LayerInfo { name, path: p, layer })
                })
                .collect();
            Ok(Ufo { layers: layers? })
        }
    }

    /// Returns the first layer matching a predicate. The predicate takes a
    /// `LayerInfo` struct, which includes the layer's name and path as well
    /// as the layer itself.
    pub fn find_layer<P>(&mut self, mut predicate: P) -> Option<&mut Layer>
    where
        P: FnMut(&LayerInfo) -> bool,
    {
        self.layers.iter_mut().find(|l| predicate(l)).map(|l| &mut l.layer)
    }

    /// Returns an iterator over all layers in this font object.
    pub fn iter(&self) -> impl Iterator<Item = &LayerInfo> {
        self.layers.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading() {
        let path = "testdata/mutatorSans/MutatorSansLightWide.ufo";
        let mut font_obj = Ufo::load(path).unwrap();
        assert_eq!(font_obj.iter().count(), 2);
        font_obj
            .find_layer(|l| l.path.to_str() == Some("glyphs.background"))
            .expect("missing layer");
    }
}
