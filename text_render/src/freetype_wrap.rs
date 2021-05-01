// This module is a quick safe wrapper around `freetype`.
use std::fmt;
use std::rc::Rc;
use std::path::Path;
use std::mem::MaybeUninit;
use std::num::NonZeroU32;
use std::error::Error as StdError;

use freetype_sys::*;

#[derive(Debug)]
pub enum Error {
    // Freetype's system errors
    ArrayTooLarge,
    BadArgument,	
    BbxTooBig,	
    CMapTableMissing,
    CannotOpenResource,
    CannotOpenStream,
    CannotRenderGlyph,
    CodeOverflow,
    CorruptedFontGlyphs,
    CorruptedFontHeader,
    CouldNotFindContext,
    DebugOpCode,
    DivideByZero,
    ENDFInExecStream,
    ExecutionTooLong,
    HmtxTableMissing,
    HorizHeaderMissing,
    Ignore,
    InvalidArgument,
    InvalidCacheHandle,
    InvalidCharMapFormat,
    InvalidCharMapHandle,
    InvalidCharacterCode,
    InvalidCodeRange,
    InvalidComposite,
    InvalidDriverHandle,
    InvalidFaceHandle,
    InvalidFileFormat,
    InvalidFrameOperation,
    InvalidFrameRead,
    InvalidGlyphFormat,
    InvalidGlyphIndex,
    InvalidHandle,
    InvalidHorizMetrics,
    InvalidLibraryHandle,
    InvalidOffset,	
    InvalidOpcode,
    InvalidOutline,
    InvalidPPem,	
    InvalidPixelSize,
    InvalidPostTable,
    InvalidPostTableFormat,
    InvalidReference,
    InvalidSizeHandle,
    InvalidSlotHandle,
    InvalidStreamHandle,
    InvalidStreamOperation,
    InvalidStreamRead,
    InvalidStreamSeek,
    InvalidStreamSkip,
    InvalidTable,
    InvalidVersion,	
    InvalidVertMetrics,
    LocationsMissing,
    LowerModuleVersion,	
    MissingBbxField,
    MissingCharsField,
    MissingEncodingField,
    MissingFontField,
    MissingFontboundingboxField,
    MissingModule,
    MissingProperty,
    MissingSizeField,
    MissingStartcharField,
    MissingStartfontField,
    NameTableMissing,
    NestedDEFS,
    NestedFrameAccess,
    NoUnicodeGlyphName,
    OutOfMemory,
    PostTableMissing,
    RasterCorrupted,
    RasterNegativeHeight,
    RasterOverflow,
    RasterUninitialized,
    StackOverflow,
    StackUnderflow,
    SyntaxError,
    TableMissing,
    TooFewArguments,
    TooManyCaches,
    TooManyDrivers,
    TooManyExtensions,
    TooManyFunctionDefs,
    TooManyHints,
    TooManyInstructionDefs,
    UnimplementedFeature,
    UnknownFileFormat,
    UnlistedObject,
    // Additional errors
    GlyphNotFound,
}

// TODO build script?
impl Error {
    fn from_raw(errno : FT_Error) -> Self {
        // Allow, because freetype is freetype :p
        #[allow(non_upper_case_globals)]
        match errno {
            FT_Err_Ok => panic!("`Ok` is not an error"),
            FT_Err_Bad_Argument => Error::BadArgument,
            FT_Err_Bbx_Too_Big => Error::BbxTooBig,
            FT_Err_CMap_Table_Missing => Error::CMapTableMissing,
            FT_Err_Cannot_Open_Resource => Error::CannotOpenResource,
            FT_Err_Cannot_Open_Stream => Error::CannotOpenStream,
            FT_Err_Cannot_Render_Glyph => Error::CannotRenderGlyph,
            FT_Err_Code_Overflow => Error::CodeOverflow,
            FT_Err_Corrupted_Font_Glyphs => Error::CorruptedFontGlyphs,
            FT_Err_Corrupted_Font_Header => Error::CorruptedFontHeader,
            FT_Err_Could_Not_Find_Context => Error::CouldNotFindContext,
            FT_Err_Debug_OpCode => Error::DebugOpCode,
            FT_Err_Divide_By_Zero => Error::DivideByZero,
            FT_Err_ENDF_In_Exec_Stream => Error::ENDFInExecStream,
            FT_Err_Execution_Too_Long => Error::ExecutionTooLong,
            FT_Err_Hmtx_Table_Missing => Error::HmtxTableMissing,
            FT_Err_Horiz_Header_Missing => Error::HorizHeaderMissing,
            FT_Err_Ignore => Error::Ignore,
            FT_Err_Invalid_Argument => Error::InvalidArgument,
            FT_Err_Invalid_Cache_Handle => Error::InvalidCacheHandle,
            FT_Err_Invalid_CharMap_Format => Error::InvalidCharMapFormat,
            FT_Err_Invalid_CharMap_Handle => Error::InvalidCharMapHandle,
            FT_Err_Invalid_Character_Code => Error::InvalidCharacterCode,
            FT_Err_Invalid_CodeRange => Error::InvalidCodeRange,
            FT_Err_Invalid_Composite => Error::InvalidComposite,
            FT_Err_Invalid_Driver_Handle => Error::InvalidDriverHandle,
            FT_Err_Invalid_Face_Handle => Error::InvalidFaceHandle,
            FT_Err_Invalid_File_Format => Error::InvalidFileFormat,
            FT_Err_Invalid_Frame_Operation => Error::InvalidFrameOperation,
            FT_Err_Invalid_Frame_Read => Error::InvalidFrameRead,
            FT_Err_Invalid_Glyph_Format => Error::InvalidGlyphFormat,
            FT_Err_Invalid_Glyph_Index => Error::InvalidGlyphIndex,
            FT_Err_Invalid_Handle => Error::InvalidHandle,
            FT_Err_Invalid_Horiz_Metrics => Error::InvalidHorizMetrics,
            FT_Err_Invalid_Library_Handle => Error::InvalidLibraryHandle,
            FT_Err_Invalid_Offset => Error::InvalidOffset,
            FT_Err_Invalid_Opcode => Error::InvalidOpcode,
            FT_Err_Invalid_Outline => Error::InvalidOutline,
            FT_Err_Invalid_PPem => Error::InvalidPPem,
            FT_Err_Invalid_Pixel_Size => Error::InvalidPixelSize,
            FT_Err_Invalid_Post_Table => Error::InvalidPostTable,
            FT_Err_Invalid_Post_Table_Format => Error::InvalidPostTableFormat,
            FT_Err_Invalid_Reference => Error::InvalidReference,
            FT_Err_Invalid_Size_Handle => Error::InvalidSizeHandle,
            FT_Err_Invalid_Slot_Handle => Error::InvalidSlotHandle,
            FT_Err_Invalid_Stream_Handle => Error::InvalidStreamHandle,
            FT_Err_Invalid_Stream_Operation => Error::InvalidStreamOperation,
            FT_Err_Invalid_Stream_Read => Error::InvalidStreamRead,
            FT_Err_Invalid_Stream_Seek => Error::InvalidStreamSeek,
            FT_Err_Invalid_Stream_Skip => Error::InvalidStreamSkip,
            FT_Err_Invalid_Table => Error::InvalidTable,
            FT_Err_Invalid_Version => Error::InvalidVersion,
            FT_Err_Invalid_Vert_Metrics => Error::InvalidVertMetrics,
            FT_Err_Locations_Missing => Error::LocationsMissing,
            FT_Err_Lower_Module_Version => Error::LowerModuleVersion,
            FT_Err_Missing_Bbx_Field => Error::MissingBbxField,
            FT_Err_Missing_Chars_Field => Error::MissingCharsField,
            FT_Err_Missing_Encoding_Field => Error::MissingEncodingField,
            FT_Err_Missing_Font_Field => Error::MissingFontField,
            FT_Err_Missing_Fontboundingbox_Field => Error::MissingFontboundingboxField,
            FT_Err_Missing_Module => Error::MissingModule,
            FT_Err_Missing_Property => Error::MissingProperty,
            FT_Err_Missing_Size_Field => Error::MissingSizeField,
            FT_Err_Missing_Startchar_Field => Error::MissingStartcharField,
            FT_Err_Missing_Startfont_Field => Error::MissingStartfontField,
            FT_Err_Name_Table_Missing => Error::NameTableMissing,
            FT_Err_Nested_DEFS => Error::NestedDEFS,
            FT_Err_Nested_Frame_Access => Error::NestedFrameAccess,
            FT_Err_No_Unicode_Glyph_Name => Error::NoUnicodeGlyphName,
            FT_Err_Out_Of_Memory => Error::OutOfMemory,
            FT_Err_Post_Table_Missing => Error::PostTableMissing,
            FT_Err_Raster_Corrupted => Error::RasterCorrupted,
            FT_Err_Raster_Negative_Height => Error::RasterNegativeHeight,
            FT_Err_Raster_Overflow => Error::RasterOverflow,
            FT_Err_Raster_Uninitialized => Error::RasterUninitialized,
            FT_Err_Stack_Overflow => Error::StackOverflow,
            FT_Err_Stack_Underflow => Error::StackUnderflow,
            FT_Err_Syntax_Error => Error::SyntaxError,
            FT_Err_Table_Missing => Error::TableMissing,
            FT_Err_Too_Few_Arguments => Error::TooFewArguments,
            FT_Err_Too_Many_Caches => Error::TooManyCaches,
            FT_Err_Too_Many_Drivers => Error::TooManyDrivers,
            FT_Err_Too_Many_Extensions => Error::TooManyExtensions,
            FT_Err_Too_Many_Function_Defs => Error::TooManyFunctionDefs,
            FT_Err_Too_Many_Hints => Error::TooManyHints,
            FT_Err_Too_Many_Instruction_Defs => Error::TooManyInstructionDefs,
            FT_Err_Unimplemented_Feature => Error::UnimplementedFeature,
            FT_Err_Unknown_File_Format => Error::UnknownFileFormat,
            FT_Err_Unlisted_Object => Error::UnlistedObject,
            _ => panic!("Unknown error code"),
        }
    }
}

struct LibContainer {
    inner : FT_Library,
}

impl LibContainer {
    // TODO make an `InitError` error enum
    fn new() -> Result<Self, FT_Error> {
        let status;
        let inner = unsafe {
            let mut val = MaybeUninit::uninit();
            status = FT_Init_FreeType(val.as_mut_ptr());
            val.assume_init()
        };

        if status != 0 { return Err(status) }
        
        Ok(
            LibContainer {
                inner,
            }
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for Error {}

impl Drop for LibContainer {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(FT_Done_FreeType(self.inner), 0);
        }
    }
}

type LibHandle = Rc<LibContainer>;

pub struct Lib {
    inner : LibHandle,
}

impl Lib {
    pub fn new() -> Result<Self, Error> {
        Ok(
            Lib {
                inner : 
                    LibContainer::new()
                    .map(|x| Rc::new(x))
                    .map_err(|e| Error::from_raw(e))?
                ,
            }
        )
    }
}

pub struct GlyphImage {
    pub width : u64,
    pub height : u64,
    pub bearing : (i32, i32),
    pub advance : u64,
    pub data : Vec<Vec<u8>>,
}

impl GlyphImage {}

pub struct Face {
    // FreeType doesn't like when you free data of the file
    // so we will keep it alive as long as the face is alive.
    // Otherwise we'll get a segfault.
    data : Vec<u8>,
    inner : FT_Face,
    lib_handle : LibHandle,
}

impl Face {
    // Quickly load something into memory
    fn load<P : AsRef<Path>>(lib : &Lib, p : P) -> Result<Self, Error> {
        let bytes = std::fs::read(p).unwrap();

        let status;
        let inner = unsafe {
            let mut val = MaybeUninit::uninit();
            // I don't use `FT_New_Face`, because
            // it turned out that it's quite hard to
            // make it work on Windows :/
            status = 
                FT_New_Memory_Face(
                    lib.inner.inner, 
                    bytes.as_ptr(),
                    bytes.len() as FT_Long,
                    0, 
                    val.as_mut_ptr()
                )
            ;
            val.assume_init()
        };

        if status != 0 { 
            // TODO do you have to free the face if you failed
            // to `new` it?
            return Err(Error::from_raw(status))
        }
        
        Ok(
            Face {
                data : bytes,
                inner,
                lib_handle : lib.inner.clone(),
            }
        )
    }

    pub fn new<P : AsRef<Path>>(
        lib : &Lib, 
        width : Option<NonZeroU32>, 
        height : Option<NonZeroU32>,
        p : P
    ) -> Result<Self, Error> {
        // Load the face separately. Because all the unsafe stuff
        // must get RAII'd ASAP. We don't want to leak for no reason, do we?
        let me = Face::load(lib, p)?;
        
        // Set pixel size
        let mut status =
            unsafe {
                FT_Set_Pixel_Sizes(
                    me.inner,
                    width.map(|n| n.get()).unwrap_or(0),
                    height.map(|n| n.get()).unwrap_or(0),
                )
            }
        ;
        if status != 0 { return Err(Error::from_raw(status)); }

        // Try to pick Rust's charmap --- Unicode
        status = unsafe { FT_Select_Charmap(me.inner, FT_ENCODING_UNICODE) };
        if status != 0 { return Err(Error::from_raw(status)); }

        Ok(me)
    }

    pub fn load_char(&self, ch : char) -> Result<GlyphImage, Error> {
        // 1. Get the glyph index
        // I do not use `FT_Load_Char`, because it silently gives us
        // an error-place-holder glyph. A more idiomatic way to handle
        // that for us would be to return an error code.
        // TODO check what happens when the placeholder char gets passed
        let glyph_index = 
            unsafe {
                FT_Get_Char_Index(
                    self.inner,
                    ch.into(),
                )
            }
        ;
        if glyph_index == 0 { return Err(Error::GlyphNotFound); }

        // 2. Load it
        let status = 
            unsafe {
                FT_Load_Glyph(
                    self.inner,
                    glyph_index,
                    FT_LOAD_RENDER,
                )
            }
        ;
        if status != 0 { return Err(Error::from_raw(status)) }

        // 3. Read the glyph into memory
        
        // Fetch bitmap
        let glyph_bitmap = 
            unsafe {
                &(*(*self.inner).glyph).bitmap
            }
        ;
        // These asserts exist, because
        // freetype can potentially make `width`
        // and `rows` negative. In that case I
        // don't really know what to do, so I'd
        // panic.
        assert!(glyph_bitmap.width > 0); 
        assert!(glyph_bitmap.rows > 0);

        // Fetch the dimensions
        let (width, height) = (glyph_bitmap.width as u64, glyph_bitmap.rows as u64);
        let len = (width * height) as usize;

        // Allocate it zeroed
        let mut luma = vec![0; len];

        // Write the data
        unsafe { glyph_bitmap.buffer.copy_to(luma.as_mut_ptr(), len); }

        // Shove it all into a multi-vec
        let data =
            luma
            .chunks(width as usize)
            .map(|row| { assert!(row.len() as u64 == width); row.to_owned() })
            .collect::<Vec<_>>()
        ; assert!(data.len() as u64 == height);

        // Fetch more glyph metrics
        let (bearing, advance) = 
            unsafe {
                let glyph = &(*(*self.inner).glyph);
                assert!(glyph.advance.x >= 0);
                ((glyph.bitmap_left, glyph.bitmap_top), glyph.advance.x as u64)
            }
        ;

        // Construct the image
        Ok(
            GlyphImage {
                width,
                height,
                data,
                bearing,
                advance,
            }
        )
    }
}

impl Drop for Face {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(FT_Done_Face(self.inner), 0);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library() {
        let _ = Lib::new().unwrap();
    }

    #[test]
    fn test_face() {
        let lib = Lib::new().unwrap();
        let face = 
            Face::new(
                &lib, 
                NonZeroU32::new(0), NonZeroU32::new(48),
                "fonts/OpenSans-Regular.ttf"
            ).unwrap()
        ;
    }

    #[test]
    fn test_load_basic_charset() {
        let lib = Lib::new().unwrap();
        let face = 
            Face::new(
                &lib, 
                NonZeroU32::new(0), NonZeroU32::new(48),
                "fonts/OpenSans-Regular.ttf"
            ).unwrap()
        ;

        let arr = 
        [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
            'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
            'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
            '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '+', '=',
            '[', ']', '{', '}', ':', ';', '\'', '"', ',', '.', '/', '<', '>', '?',
            '\\', '|', '`', '~'
        ];

        for x in arr.iter() { face.load_char(*x).unwrap(); }
    }
}
