//! Data related to individual glyphs.

mod parse;
mod serialize;
#[cfg(test)]
mod tests;

use crate::Error;
use std::path::{Path, PathBuf};

//FIXME: actually load the 'lib' data
type Plist = ();

/// A glyph, loaded from a [.glif file][glif].
///
/// [glif]: http://unifiedfontobject.org/versions/ufo3/glyphs/glif/
#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub name: String,
    pub format: GlifVersion,
    pub advance: Option<Advance>,
    pub codepoints: Option<Vec<char>>,
    pub note: Option<String>,
    pub guidelines: Option<Vec<Guideline>>,
    pub anchors: Option<Vec<Anchor>>,
    pub outline: Option<Outline>,
    pub image: Option<Image>,
    pub lib: Option<Plist>,
}

impl Glyph {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let data = std::fs::read(path.as_ref())?;
        parse::parse_glyph(&data)
    }

    #[doc(hidden)]
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let data = self.encode_xml()?;
        std::fs::write(path, &data)?;
        Ok(())
    }

    /// Create a new glyph with the given name.
    pub fn new_named<S: Into<String>>(name: S) -> Self {
        Glyph::new(name.into(), GlifVersion::V2)
    }

    pub(crate) fn new(name: String, format: GlifVersion) -> Self {
        Glyph {
            name,
            format,
            advance: None,
            codepoints: None,
            note: None,
            guidelines: None,
            anchors: None,
            outline: None,
            image: None,
            lib: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GlifVersion {
    V1 = 1,
    V2 = 2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Advance {
    Width(f32),
    Height(f32),
}

/// Identifiers are optional attributes of several objects in the UFO.
/// These identifiers are required to be unique within certain contexts
/// as defined on a per object basis throughout this specification.
/// Identifiers are specified as a string between one and 100 characters long.
/// All characters must be in the printable ASCII range, 0x20 to 0x7E.
#[derive(Debug, Clone, PartialEq)]
pub struct Identifier(pub(crate) String);

/// A guideline associated with a glyph.
#[derive(Debug, Clone, PartialEq)]
pub struct Guideline {
    /// The line itself.
    pub line: Line,
    /// An arbitrary name for the guideline.
    pub name: Option<String>,
    /// The color of the line.
    pub color: Option<Color>,
    /// Unique identifier for the guideline. This attribute is not required
    /// and should only be added to guidelines as needed.
    pub identifier: Option<Identifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    /// A vertical line, passing through a given `x` coordinate.
    Vertical(f32),
    /// A horizontal line, passing through a given `y` coordinate.
    Horizontal(f32),
    /// An angled line passing through `(x, y)` at `degrees` degrees counteer-clockwise
    /// to the horizontal.
    Angle { x: f32, y: f32, degrees: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Anchor {
    pub x: f32,
    pub y: f32,
    /// An arbitrary name for the anchor.
    pub name: Option<String>,
    pub color: Option<Color>,
    pub identifier: Option<Identifier>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Outline {
    pub components: Vec<Component>,
    pub contours: Vec<Contour>,
}

/// Another glyph inserted as part of the outline.
#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    /// The name of the base glyph.
    pub base: String,
    pub transform: AffineTransform,
    pub identifier: Option<Identifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Contour {
    pub identifier: Option<Identifier>,
    pub points: Vec<ContourPoint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContourPoint {
    pub name: Option<String>,
    pub x: f32,
    pub y: f32,
    pub typ: PointType,
    pub smooth: bool,
    pub identifier: Option<Identifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointType {
    /// A point of this type must be the first in a contour. The reverse is not true:
    /// a contour does not necessarily start with a move point. When a contour
    /// does start with a move point, it signifies the beginning of an open contour.
    /// A closed contour does not start with a move and is defined as a cyclic
    /// list of points, with no predominant start point. There is always a next
    /// point and a previous point. For this purpose the list of points can be
    /// seen as endless in both directions. The actual list of points can be
    /// rotated arbitrarily (by removing the first N points and appending
    /// them at the end) while still describing the same outline.
    Move,
    /// Draw a straight line from the previous point to this point.
    /// The previous point must be a move, a line, a curve or a qcurve.
    /// It must not be an offcurve.
    Line,
    /// This point is part of a curve segment that goes up to the next point
    /// that is either a curve or a qcurve.
    OffCurve,
    /// Draw a cubic bezier curve from the last non-offcurve point to this point.
    /// The number of offcurve points can be zero, one or two.
    /// If the number of offcurve points is zero, a straight line is drawn.
    /// If it is one, a quadratic curve is drawn.
    /// If it is two, a regular cubic bezier is drawn.
    Curve,
    /// Similar to curve, but uses quadratic curves, using the TrueType
    /// “implied on-curve points” principle.
    QCurve,
}

/// Taken together in order, these fields represent an affine transformation matrix.
#[derive(Debug, Clone, PartialEq)]
pub struct AffineTransform {
    pub x_scale: f32,
    pub xy_scale: f32,
    pub yx_scale: f32,
    pub y_scale: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

impl AffineTransform {
    ///  [1 0 0 1 0 0]; the identity transformation.
    fn identity() -> Self {
        AffineTransform {
            x_scale: 1.0,
            xy_scale: 0.,
            yx_scale: 0.,
            y_scale: 1.0,
            x_offset: 0.,
            y_offset: 0.,
        }
    }
}

impl std::default::Default for AffineTransform {
    fn default() -> Self {
        Self::identity()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    /// Not an absolute / relative path, but the name of the image file.
    pub file_name: PathBuf,
    pub color: Option<Color>,
    pub transform: AffineTransform,
}
