/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method, to_camel_case, to_rust_ident, SYSTEM_FONT_LONGHANDS %>

<% data.new_style_struct("Font",
                         inherited=True) %>
<%helpers:longhand name="font-family" animatable="False" need_index="True"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-family">
    use properties::longhands::system_font::SystemFont;
    use self::computed_value::{FontFamily, FamilyName};
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;

    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use cssparser::{CssStringWriter, Parser};
        use properties::longhands::system_font::SystemFont;
        use std::fmt::{self, Write};
        use Atom;
        use style_traits::ToCss;
        pub use self::FontFamily as SingleComputedValue;

        #[derive(Debug, PartialEq, Eq, Clone, Hash)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub enum FontFamily {
            FamilyName(FamilyName),
            Generic(Atom),
        }

        #[derive(Debug, PartialEq, Eq, Clone, Hash)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub struct FamilyName(pub Atom);

        impl FontFamily {
            #[inline]
            pub fn atom(&self) -> &Atom {
                match *self {
                    FontFamily::FamilyName(ref name) => &name.0,
                    FontFamily::Generic(ref name) => name,
                }
            }

            #[inline]
            #[cfg(not(feature = "gecko"))] // Gecko can't borrow atoms as UTF-8.
            pub fn name(&self) -> &str {
                self.atom()
            }

            #[cfg(not(feature = "gecko"))] // Gecko can't borrow atoms as UTF-8.
            pub fn from_atom(input: Atom) -> FontFamily {
                match input {
                    atom!("serif") |
                    atom!("sans-serif") |
                    atom!("cursive") |
                    atom!("fantasy") |
                    atom!("monospace") => {
                        return FontFamily::Generic(input)
                    }
                    _ => {}
                }
                match_ignore_ascii_case! { &input,
                    "serif" => return FontFamily::Generic(atom!("serif")),
                    "sans-serif" => return FontFamily::Generic(atom!("sans-serif")),
                    "cursive" => return FontFamily::Generic(atom!("cursive")),
                    "fantasy" => return FontFamily::Generic(atom!("fantasy")),
                    "monospace" => return FontFamily::Generic(atom!("monospace")),
                    _ => {}
                }
                FontFamily::FamilyName(FamilyName(input))
            }

            /// Parse a font-family value
            pub fn parse(input: &mut Parser) -> Result<Self, ()> {
                if let Ok(value) = input.try(|input| input.expect_string()) {
                    return Ok(FontFamily::FamilyName(FamilyName(Atom::from(&*value))))
                }
                let first_ident = try!(input.expect_ident());

                // FIXME(bholley): The fast thing to do here would be to look up the
                // string (as lowercase) in the static atoms table. We don't have an
                // API to do that yet though, so we do the simple thing for now.
                let mut css_wide_keyword = false;
                match_ignore_ascii_case! { &first_ident,
                    "serif" => return Ok(FontFamily::Generic(atom!("serif"))),
                    "sans-serif" => return Ok(FontFamily::Generic(atom!("sans-serif"))),
                    "cursive" => return Ok(FontFamily::Generic(atom!("cursive"))),
                    "fantasy" => return Ok(FontFamily::Generic(atom!("fantasy"))),
                    "monospace" => return Ok(FontFamily::Generic(atom!("monospace"))),
                    % if product == "gecko":
                        "-moz-fixed" => return Ok(FontFamily::Generic(atom!("-moz-fixed"))),
                    % endif

                    // https://drafts.csswg.org/css-fonts/#propdef-font-family
                    // "Font family names that happen to be the same as a keyword value
                    //  (`inherit`, `serif`, `sans-serif`, `monospace`, `fantasy`, and `cursive`)
                    //  must be quoted to prevent confusion with the keywords with the same names.
                    //  The keywords ‘initial’ and ‘default’ are reserved for future use
                    //  and must also be quoted when used as font names.
                    //  UAs must not consider these keywords as matching the <family-name> type."
                    "inherit" => css_wide_keyword = true,
                    "initial" => css_wide_keyword = true,
                    "unset" => css_wide_keyword = true,
                    "default" => css_wide_keyword = true,
                    _ => {}
                }

                let mut value = first_ident.into_owned();
                // These keywords are not allowed by themselves.
                // The only way this value can be valid with with another keyword.
                if css_wide_keyword {
                    let ident = input.expect_ident()?;
                    value.push_str(" ");
                    value.push_str(&ident);
                }
                while let Ok(ident) = input.try(|input| input.expect_ident()) {
                    value.push_str(" ");
                    value.push_str(&ident);
                }
                Ok(FontFamily::FamilyName(FamilyName(Atom::from(value))))
            }
        }

        impl ToCss for FamilyName {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                dest.write_char('"')?;
                write!(CssStringWriter::new(dest), "{}", self.0)?;
                dest.write_char('"')
            }
        }

        impl ToCss for FontFamily {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    FontFamily::FamilyName(ref name) => name.to_css(dest),

                    // All generic values accepted by the parser are known to not require escaping.
                    FontFamily::Generic(ref name) => {
                        % if product == "gecko":
                            // We should treat -moz-fixed as monospace
                            if name == &atom!("-moz-fixed") {
                                return write!(dest, "monospace");
                            }
                        % endif

                        write!(dest, "{}", name)
                    },
                }
            }
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.0.iter();
                try!(iter.next().unwrap().to_css(dest));
                for family in iter {
                    try!(dest.write_str(", "));
                    try!(family.to_css(dest));
                }
                Ok(())
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<FontFamily>);
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![FontFamily::Generic(atom!("serif"))])
    }

    /// <family-name>#
    /// <family-name> = <string> | [ <ident>+ ]
    /// TODO: <generic-family>
    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        SpecifiedValue::parse(input)
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SpecifiedValue {
        Values(Vec<FontFamily>),
        System(SystemFont),
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;
        fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
            match *self {
                SpecifiedValue::Values(ref v) => computed_value::T(v.clone()),
                SpecifiedValue::System(_) => cx.style.cached_system_font.as_ref().unwrap().font_family.clone(),
            }
        }
        fn from_computed_value(other: &computed_value::T) -> Self {
            SpecifiedValue::Values(other.0.clone())
        }
    }

    impl SpecifiedValue {
        pub fn system_font(f: SystemFont) -> Self {
            SpecifiedValue::System(f)
        }
        pub fn get_system(&self) -> Option<SystemFont> {
            if let SpecifiedValue::System(s) = *self {
                Some(s)
            } else {
                None
            }
        }

        pub fn parse(input: &mut Parser) -> Result<Self, ()> {
            input.parse_comma_separated(|input| FontFamily::parse(input)).map(SpecifiedValue::Values)
        }
    }


    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Values(ref v) => {
                    let mut iter = v.iter();
                    try!(iter.next().unwrap().to_css(dest));
                    for family in iter {
                        try!(dest.write_str(", "));
                        try!(family.to_css(dest));
                    }
                    Ok(())
                }
                _ => Ok(())
            }
        }
    }

    /// `FamilyName::parse` is based on `FontFamily::parse` and not the other way around
    /// because we want the former to exclude generic family keywords.
    impl Parse for FamilyName {
        fn parse(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
            match FontFamily::parse(input) {
                Ok(FontFamily::FamilyName(name)) => Ok(name),
                Ok(FontFamily::Generic(_)) |
                Err(()) => Err(())
            }
        }
    }
</%helpers:longhand>

${helpers.single_keyword_system("font-style",
                      "normal italic oblique",
                      gecko_constant_prefix="NS_FONT_STYLE",
                      gecko_ffi_name="mFont.style",
                      spec="https://drafts.csswg.org/css-fonts/#propdef-font-style",
                      animatable=False)}

${helpers.single_keyword("font-variant",
                         "normal small-caps",
                         spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant",
                         animatable=False)}


<% font_variant_caps_custom_consts= { "small-caps": "SMALLCAPS",
                                      "all-small": "ALLSMALL",
                                      "petite-caps": "PETITECAPS",
                                      "all-petite": "ALLPETITE",
                                      "titling-caps": "TITLING" } %>

${helpers.single_keyword_system("font-variant-caps",
                                "normal small-caps all-small petite-caps unicase titling-caps",
                                gecko_constant_prefix="NS_FONT_VARIANT_CAPS",
                                gecko_ffi_name="mFont.variantCaps",
                                products="gecko",
                                spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-caps",
                                custom_consts=font_variant_caps_custom_consts,
                                animatable=False)}

<%helpers:longhand name="font-weight" need_clone="True" animatable="True"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-weight">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use properties::longhands::system_font::SystemFont;

    no_viewport_percentage!(SpecifiedValue);

    #[derive(Debug, Clone, PartialEq, Eq, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Normal,
        Bold,
        Bolder,
        Lighter,
        % for weight in range(100, 901, 100):
            Weight${weight},
        % endfor
        System(SystemFont),
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Normal => dest.write_str("normal"),
                SpecifiedValue::Bold => dest.write_str("bold"),
                SpecifiedValue::Bolder => dest.write_str("bolder"),
                SpecifiedValue::Lighter => dest.write_str("lighter"),
                % for weight in range(100, 901, 100):
                    SpecifiedValue::Weight${weight} => dest.write_str("${weight}"),
                % endfor
                SpecifiedValue::System(_) => Ok(())
            }
        }
    }

    /// normal | bold | bolder | lighter | 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        input.try(|input| {
            match_ignore_ascii_case! { &try!(input.expect_ident()),
                "normal" => Ok(SpecifiedValue::Normal),
                "bold" => Ok(SpecifiedValue::Bold),
                "bolder" => Ok(SpecifiedValue::Bolder),
                "lighter" => Ok(SpecifiedValue::Lighter),
                _ => Err(())
            }
        }).or_else(|()| {
            match try!(input.expect_integer()) {
                100 => Ok(SpecifiedValue::Weight100),
                200 => Ok(SpecifiedValue::Weight200),
                300 => Ok(SpecifiedValue::Weight300),
                400 => Ok(SpecifiedValue::Weight400),
                500 => Ok(SpecifiedValue::Weight500),
                600 => Ok(SpecifiedValue::Weight600),
                700 => Ok(SpecifiedValue::Weight700),
                800 => Ok(SpecifiedValue::Weight800),
                900 => Ok(SpecifiedValue::Weight900),
                _ => Err(())
            }
        })
    }

    impl SpecifiedValue {
        pub fn system_font(f: SystemFont) -> Self {
            SpecifiedValue::System(f)
        }
        pub fn get_system(&self) -> Option<SystemFont> {
            if let SpecifiedValue::System(s) = *self {
                Some(s)
            } else {
                None
            }
        }
    }

    /// Used in @font-face, where relative keywords are not allowed.
    impl Parse for computed_value::T {
        fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
            match parse(context, input)? {
                % for weight in range(100, 901, 100):
                    SpecifiedValue::Weight${weight} => Ok(computed_value::T::Weight${weight}),
                % endfor
                SpecifiedValue::Normal => Ok(computed_value::T::Weight400),
                SpecifiedValue::Bold => Ok(computed_value::T::Weight700),
                SpecifiedValue::Bolder |
                SpecifiedValue::Lighter |
                SpecifiedValue::System(..) => Err(()),
            }
        }
    }

    pub mod computed_value {
        use std::fmt;
        #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        #[repr(u16)]
        pub enum T {
            % for weight in range(100, 901, 100):
                Weight${weight} = ${weight},
            % endfor
        }
        impl T {
            #[inline]
            pub fn is_bold(self) -> bool {
                match self {
                    T::Weight900 | T::Weight800 |
                    T::Weight700 | T::Weight600 => true,
                    _ => false
                }
            }

            /// Obtain a Servo computed value from a Gecko computed font-weight
            pub unsafe fn from_gecko_weight(weight: u16) -> Self {
                use std::mem::transmute;
                debug_assert!(weight >= 100);
                debug_assert!(weight <= 900);
                debug_assert!(weight % 10 == 0);
                transmute(weight)
            }
        }
    }
    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                % for weight in range(100, 901, 100):
                    computed_value::T::Weight${weight} => dest.write_str("${weight}"),
                % endfor
            }
        }
    }
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::Weight400  // normal
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Normal
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            match *self {
                % for weight in range(100, 901, 100):
                    SpecifiedValue::Weight${weight} => computed_value::T::Weight${weight},
                % endfor
                SpecifiedValue::Normal => computed_value::T::Weight400,
                SpecifiedValue::Bold => computed_value::T::Weight700,
                SpecifiedValue::Bolder => match context.inherited_style().get_font().clone_font_weight() {
                    computed_value::T::Weight100 => computed_value::T::Weight400,
                    computed_value::T::Weight200 => computed_value::T::Weight400,
                    computed_value::T::Weight300 => computed_value::T::Weight400,
                    computed_value::T::Weight400 => computed_value::T::Weight700,
                    computed_value::T::Weight500 => computed_value::T::Weight700,
                    computed_value::T::Weight600 => computed_value::T::Weight900,
                    computed_value::T::Weight700 => computed_value::T::Weight900,
                    computed_value::T::Weight800 => computed_value::T::Weight900,
                    computed_value::T::Weight900 => computed_value::T::Weight900,
                },
                SpecifiedValue::Lighter => match context.inherited_style().get_font().clone_font_weight() {
                    computed_value::T::Weight100 => computed_value::T::Weight100,
                    computed_value::T::Weight200 => computed_value::T::Weight100,
                    computed_value::T::Weight300 => computed_value::T::Weight100,
                    computed_value::T::Weight400 => computed_value::T::Weight100,
                    computed_value::T::Weight500 => computed_value::T::Weight100,
                    computed_value::T::Weight600 => computed_value::T::Weight400,
                    computed_value::T::Weight700 => computed_value::T::Weight400,
                    computed_value::T::Weight800 => computed_value::T::Weight700,
                    computed_value::T::Weight900 => computed_value::T::Weight700,
                },
                SpecifiedValue::System(_) => {
                    context.style.cached_system_font.as_ref().unwrap().font_weight.clone()
                }
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                % for weight in range(100, 901, 100):
                    computed_value::T::Weight${weight} => SpecifiedValue::Weight${weight},
                % endfor
            }
        }
    }
</%helpers:longhand>

<%helpers:longhand name="font-size" need_clone="True" animatable="True"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-size">
    use app_units::Au;
    use std::fmt;
    use style_traits::ToCss;
    use values::{FONT_MEDIUM_PX, HasViewportPercentage};
    use values::specified::{LengthOrPercentage, Length, NoCalcLength, Percentage};
    use properties::longhands::system_font::SystemFont;

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Length(ref lop) => lop.to_css(dest),
                SpecifiedValue::Keyword(kw) => kw.to_css(dest),
                SpecifiedValue::System(_) => Ok(()),
            }
        }
    }

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                SpecifiedValue::Length(ref lop) => lop.has_viewport_percentage(),
                _ => false
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Length(specified::LengthOrPercentage),
        Keyword(KeywordSize),
        System(SystemFont)
    }

    pub mod computed_value {
        use app_units::Au;
        pub type T = Au;
    }

    /// CSS font keywords
    #[derive(Debug, Copy, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum KeywordSize {
        XXSmall = 0,
        XSmall = 1,
        Small = 2,
        Medium = 3,
        Large = 4,
        XLarge = 5,
        XXLarge = 6,
        // This is not a real font keyword and will not parse
        // HTML font-size 7 corresponds to this value
        XXXLarge = 7,
    }

    pub use self::KeywordSize::*;

    impl KeywordSize {
        pub fn parse(input: &mut Parser) -> Result<Self, ()> {
            Ok(match_ignore_ascii_case! {&*input.expect_ident()?,
                "xx-small" => XXSmall,
                "x-small" => XSmall,
                "small" => Small,
                "medium" => Medium,
                "large" => Large,
                "x-large" => XLarge,
                "xx-large" => XXLarge,
                _ => return Err(())
            })
        }
    }

    impl Default for KeywordSize {
        fn default() -> Self {
            Medium
        }
    }

    impl ToCss for KeywordSize {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            dest.write_str(match *self {
                XXSmall => "xx-small",
                XSmall => "x-small",
                Small => "small",
                Medium => "medium",
                Large => "large",
                XLarge => "x-large",
                XXLarge => "xx-large",
                XXXLarge => "",
            })
        }
    }

    % if product == "servo":
        impl ToComputedValue for KeywordSize {
            type ComputedValue = Au;
            #[inline]
            fn to_computed_value(&self, _: &Context) -> computed_value::T {
                // https://drafts.csswg.org/css-fonts-3/#font-size-prop
                use values::FONT_MEDIUM_PX;
                match *self {
                    XXSmall => Au::from_px(FONT_MEDIUM_PX) * 3 / 5,
                    XSmall => Au::from_px(FONT_MEDIUM_PX) * 3 / 4,
                    Small => Au::from_px(FONT_MEDIUM_PX) * 8 / 9,
                    Medium => Au::from_px(FONT_MEDIUM_PX),
                    Large => Au::from_px(FONT_MEDIUM_PX) * 6 / 5,
                    XLarge => Au::from_px(FONT_MEDIUM_PX) * 3 / 2,
                    XXLarge => Au::from_px(FONT_MEDIUM_PX) * 2,
                    XXXLarge => Au::from_px(FONT_MEDIUM_PX) * 3,
                }
            }

            #[inline]
            fn from_computed_value(_: &computed_value::T) -> Self {
                unreachable!()
            }
        }
    % else:
        impl ToComputedValue for KeywordSize {
            type ComputedValue = Au;
            #[inline]
            fn to_computed_value(&self, cx: &Context) -> computed_value::T {
                use gecko_bindings::bindings::Gecko_nsStyleFont_GetBaseSize;
                use values::specified::length::au_to_int_px;
                // Data from nsRuleNode.cpp in Gecko
                // Mapping from base size and HTML size to pixels
                // The first index is (base_size - 9), the second is the
                // HTML size. "0" is CSS keyword xx-small, not HTML size 0,
                // since HTML size 0 is the same as 1.
                //
                //  xxs   xs      s      m     l      xl     xxl   -
                //  -     0/1     2      3     4      5      6     7
                static FONT_SIZE_MAPPING: [[i32; 8]; 8] = [
                    [9,    9,     9,     9,    11,    14,    18,    27],
                    [9,    9,     9,    10,    12,    15,    20,    30],
                    [9,    9,    10,    11,    13,    17,    22,    33],
                    [9,    9,    10,    12,    14,    18,    24,    36],
                    [9,   10,    12,    13,    16,    20,    26,    39],
                    [9,   10,    12,    14,    17,    21,    28,    42],
                    [9,   10,    13,    15,    18,    23,    30,    45],
                    [9,   10,    13,    16,    18,    24,    32,    48]
                ];

                static FONT_SIZE_FACTORS: [i32; 8] = [60, 75, 89, 100, 120, 150, 200, 300];

                // XXXManishearth handle quirks mode

                let base_size = unsafe {
                    Gecko_nsStyleFont_GetBaseSize(cx.style().get_font().gecko(),
                                                  &*cx.device.pres_context)
                };
                let base_size_px = au_to_int_px(base_size as f32);
                let html_size = *self as usize;
                if base_size_px >= 9 && base_size_px <= 16 {
                    Au::from_px(FONT_SIZE_MAPPING[(base_size_px - 9) as usize][html_size])
                } else {
                    Au(FONT_SIZE_FACTORS[html_size] * base_size / 100)
                }
            }

            #[inline]
            fn from_computed_value(_: &computed_value::T) -> Self {
                unreachable!()
            }
        }
    % endif

    impl SpecifiedValue {
        pub fn from_html_size(size: u8) -> Self {
            SpecifiedValue::Keyword(match size {
                0 | 1 => XSmall,
                2 => Small,
                3 => Medium,
                4 => Large,
                5 => XLarge,
                6 => XXLarge,
                _ => XXXLarge,
            })
        }
    }

    pub static MEDIUM_DECLARATION: PropertyDeclaration =
        PropertyDeclaration::FontSize(DeclaredValue::Value(
            SpecifiedValue::Keyword(Medium)
    ));

    #[inline]
    #[allow(missing_docs)]
    pub fn get_initial_value() -> computed_value::T {
        Au::from_px(FONT_MEDIUM_PX)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Keyword(Medium)
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            match *self {
                SpecifiedValue::Length(LengthOrPercentage::Length(
                        NoCalcLength::FontRelative(value))) => {
                    value.to_computed_value(context, /* use inherited */ true)
                }
                SpecifiedValue::Length(LengthOrPercentage::Length(
                        NoCalcLength::ServoCharacterWidth(value))) => {
                    value.to_computed_value(context.inherited_style().get_font().clone_font_size())
                }
                SpecifiedValue::Length(LengthOrPercentage::Length(ref l)) => {
                    l.to_computed_value(context)
                }
                SpecifiedValue::Length(LengthOrPercentage::Percentage(Percentage(value))) => {
                    context.inherited_style().get_font().clone_font_size().scale_by(value)
                }
                SpecifiedValue::Length(LengthOrPercentage::Calc(ref calc)) => {
                    let calc = calc.to_computed_value(context);
                    calc.length() + context.inherited_style().get_font().clone_font_size()
                                           .scale_by(calc.percentage())
                }
                SpecifiedValue::Keyword(ref key) => {
                    key.to_computed_value(context)
                }
                SpecifiedValue::System(_) => {
                    context.style.cached_system_font.as_ref().unwrap().font_size
                }
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
                SpecifiedValue::Length(LengthOrPercentage::Length(
                        ToComputedValue::from_computed_value(computed)
                ))
        }
    }
    /// <length> | <percentage> | <absolute-size> | <relative-size>
    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use values::specified::{FontRelativeLength, Length, LengthOrPercentage, NoCalcLength};
        if let Ok(lop) = input.try(specified::LengthOrPercentage::parse_non_negative) {
            Ok(SpecifiedValue::Length(lop))
        } else if let Ok(kw) = input.try(KeywordSize::parse) {
            Ok(SpecifiedValue::Keyword(kw))
        } else {
            let ret = match_ignore_ascii_case! {&*input.expect_ident()?,
                "smaller" => FontRelativeLength::Em(0.85),
                "larger" => FontRelativeLength::Em(1.2),
                _ => return Err(())
            };
            Ok(SpecifiedValue::Length(NoCalcLength::FontRelative(ret).into()))
        }
    }
    impl SpecifiedValue {
        pub fn system_font(f: SystemFont) -> Self {
            SpecifiedValue::System(f)
        }
        pub fn get_system(&self) -> Option<SystemFont> {
            if let SpecifiedValue::System(s) = *self {
                Some(s)
            } else {
                None
            }
        }
    }
</%helpers:longhand>

<%helpers:longhand products="gecko" name="font-size-adjust" animatable="True"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-size-adjust">
    use properties::longhands::system_font::SystemFont;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    use values::specified::Number;

    no_viewport_percentage!(SpecifiedValue);

    #[derive(Copy, Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Value(computed_value::T),
        System(SystemFont),
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            match *self {
                SpecifiedValue::Value(t) => t,
                SpecifiedValue::System(_) => {
                    context.style.cached_system_font.as_ref().unwrap().font_size_adjust
                }
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue::Value(*computed)
        }
    }

    impl SpecifiedValue {
        pub fn system_font(f: SystemFont) -> Self {
            SpecifiedValue::System(f)
        }
        pub fn get_system(&self) -> Option<SystemFont> {
            if let SpecifiedValue::System(s) = *self {
                Some(s)
            } else {
                None
            }
        }
    }

    pub mod computed_value {
        use style_traits::ToCss;
        use std::fmt;
        use properties::animated_properties::Interpolate;
        use values::specified::Number;

        #[derive(Copy, Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            None,
            Number(Number),
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    T::None => dest.write_str("none"),
                    T::Number(number) => number.to_css(dest),
                }
            }
        }

        impl T {
            pub fn from_gecko_adjust(gecko: f32) -> Self {
                match gecko {
                    -1.0 => T::None,
                    _ => T::Number(Number(gecko)),
                }
            }
        }

        impl Interpolate for T {
            fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
                match (*self, *other) {
                    (T::Number(ref number), T::Number(ref other)) =>
                        Ok(T::Number(Number(try!(number.0.interpolate(&other.0, time))))),
                    _ => Err(()),
                }
            }
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Value(ref v) => v.to_css(dest),
                SpecifiedValue::System(_) => Ok(()),
            }
        }
    }

    #[inline] pub fn get_initial_value() -> computed_value::T {
        computed_value::T::None
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Value(computed_value::T::None)
    }

    /// none | <number>
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use values::specified::Number;

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue::Value(computed_value::T::None));
        }

        Ok(SpecifiedValue::Value(computed_value::T::Number(try!(Number::parse_non_negative(input)))))
    }
</%helpers:longhand>

<%helpers:longhand products="gecko" name="font-synthesis" animatable="False"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-synthesis">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        pub weight: bool,
        pub style: bool,
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.weight && self.style {
                dest.write_str("weight style")
            } else if self.style {
                dest.write_str("style")
            } else if self.weight {
                dest.write_str("weight")
            } else {
                dest.write_str("none")
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        SpecifiedValue { weight: true, style: true }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut result = SpecifiedValue { weight: false, style: false };
        match_ignore_ascii_case! { &try!(input.expect_ident()),
            "none" => Ok(result),
            "weight" => {
                result.weight = true;
                if input.try(|input| input.expect_ident_matching("style")).is_ok() {
                    result.style = true;
                }
                Ok(result)
            },
            "style" => {
                result.style = true;
                if input.try(|input| input.expect_ident_matching("weight")).is_ok() {
                    result.weight = true;
                }
                Ok(result)
            },
            _ => Err(())
        }
    }
</%helpers:longhand>

// FIXME: This prop should be animatable
${helpers.single_keyword_system("font-stretch",
                                "normal ultra-condensed extra-condensed condensed \
                                 semi-condensed semi-expanded expanded extra-expanded \
                                 ultra-expanded",
                                cast_to="i32",
                                gecko_ffi_name="mFont.stretch",
                                gecko_constant_prefix="NS_FONT_STRETCH",
                                cast_type='i16',
                                spec="https://drafts.csswg.org/css-fonts/#propdef-font-stretch",
                                animatable=False)}

${helpers.single_keyword_system("font-kerning",
                                "auto none normal",
                                products="gecko",
                                gecko_ffi_name="mFont.kerning",
                                gecko_constant_prefix="NS_FONT_KERNING",
                                spec="https://drafts.csswg.org/css-fonts/#propdef-font-stretch",
                                animatable=False)}

${helpers.single_keyword_system("font-variant-position",
                                "normal sub super",
                                products="gecko",
                                gecko_ffi_name="mFont.variantPosition",
                                gecko_constant_prefix="NS_FONT_VARIANT_POSITION",
                                spec="https://drafts.csswg.org/css-fonts/#propdef-font-variant-position",
                                animatable=False)}

<%helpers:longhand name="font-feature-settings" products="none" animatable="False" extra_prefixes="moz"
                   spec="https://drafts.csswg.org/css-fonts/#propdef-font-feature-settings">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use cssparser::Parser;
        use parser::{Parse, ParserContext};
        use std::fmt;
        use style_traits::ToCss;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            Normal,
            Tag(Vec<FeatureTagValue>)
        }

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct FeatureTagValue {
            pub tag: String,
            pub value: i32
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    T::Normal => dest.write_str("normal"),
                    T::Tag(ref ftvs) => {
                        let mut iter = ftvs.iter();
                        // handle head element
                        try!(iter.next().unwrap().to_css(dest));
                        // handle tail, precede each with a delimiter
                        for ftv in iter {
                            try!(dest.write_str(", "));
                            try!(ftv.to_css(dest));
                        }
                        Ok(())
                    }
                }
            }
        }

        impl ToCss for FeatureTagValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.value {
                    1 => write!(dest, "\"{}\"", self.tag),
                    0 => write!(dest, "\"{}\" off", self.tag),
                    x => write!(dest, "\"{}\" {}", self.tag, x)
                }
            }
        }

        impl Parse for FeatureTagValue {
            /// https://www.w3.org/TR/css-fonts-3/#propdef-font-feature-settings
            /// <string> [ on | off | <integer> ]
            fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
                let tag = try!(input.expect_string());

                // allowed strings of length 4 containing chars: <U+20, U+7E>
                if tag.len() != 4 ||
                   tag.chars().any(|c| c < ' ' || c > '~')
                {
                    return Err(())
                }

                if let Ok(value) = input.try(|input| input.expect_integer()) {
                    // handle integer, throw if it is negative
                    if value >= 0 {
                        Ok(FeatureTagValue { tag: tag.into_owned(), value: value })
                    } else {
                        Err(())
                    }
                } else if let Ok(_) = input.try(|input| input.expect_ident_matching("on")) {
                    // on is an alias for '1'
                    Ok(FeatureTagValue { tag: tag.into_owned(), value: 1 })
                } else if let Ok(_) = input.try(|input| input.expect_ident_matching("off")) {
                    // off is an alias for '0'
                    Ok(FeatureTagValue { tag: tag.into_owned(), value: 0 })
                } else {
                    // empty value is an alias for '1'
                    Ok(FeatureTagValue { tag:tag.into_owned(), value: 1 })
                }
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::Normal
    }

    /// normal | <feature-tag-value>#
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            Ok(computed_value::T::Normal)
        } else {
            input.parse_comma_separated(|i| computed_value::FeatureTagValue::parse(context, i))
                 .map(computed_value::T::Tag)
        }
    }
</%helpers:longhand>

// https://www.w3.org/TR/css-fonts-3/#propdef-font-language-override
<%helpers:longhand name="font-language-override" products="none" animatable="False" extra_prefixes="moz"
                   spec="https://drafts.csswg.org/css-fonts-3/#propdef-font-language-override">
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    T::Normal => dest.write_str("normal"),
                    T::Override(ref lang) => write!(dest, "\"{}\"", lang),
                }
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            Normal,
            Override(String),
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::Normal
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Normal
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            Ok(SpecifiedValue::Normal)
        } else {
            input.expect_string().map(|cow| {
                SpecifiedValue::Override(cow.into_owned())
            })
        }
    }
</%helpers:longhand>

<%helpers:longhand name="-x-lang" products="gecko" animatable="False" internal="True"
                   spec="Internal (not web-exposed)"
                   internal="True">
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;
    pub use self::computed_value::T as SpecifiedValue;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use Atom;
        use std::fmt;
        use style_traits::ToCss;

        impl ToCss for T {
            fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
                Ok(())
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Atom);
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(atom!(""))
    }

    pub fn parse(_context: &ParserContext, _input: &mut Parser) -> Result<SpecifiedValue, ()> {
        debug_assert!(false, "Should be set directly by presentation attributes only.");
        Err(())
    }
</%helpers:longhand>

% if product == "gecko":
    pub mod system_font {
        use app_units::Au;
        use cssparser::Parser;
        use properties::longhands;
        use std::hash::{Hash, Hasher};
        use values::computed::{ToComputedValue, Context};
        <%
            system_fonts = """caption icon menu message-box small-caption status-bar
                              -moz-window -moz-document -moz-workspace -moz-desktop
                              -moz-info -moz-dialog -moz-button -moz-pull-down-menu
                              -moz-list -moz-field""".split()
            kw_font_props = """font_style font_variant_caps font_stretch
                               font_kerning font_variant_position""".split()
        %>
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum SystemFont {
            % for font in system_fonts:
                ${to_camel_case(font)},
            % endfor
        }

        // ComputedValues are compared at times
        // so we need these impls. We don't want to
        // add Eq to Number (which contains a float)
        // so instead we have an eq impl which skips the
        // cached values
        impl PartialEq for ComputedSystemFont {
            fn eq(&self, other: &Self) -> bool {
                self.system_font == other.system_font
            }
        }
        impl Eq for ComputedSystemFont {}

        impl Hash for ComputedSystemFont {
            fn hash<H: Hasher>(&self, hasher: &mut H) {
                self.system_font.hash(hasher)
            }
        }

        impl ToComputedValue for SystemFont {
            type ComputedValue = ComputedSystemFont;

            fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
                use gecko_bindings::bindings;
                use gecko_bindings::structs::{LookAndFeel_FontID, nsFont};
                use std::mem;

                let mut system: nsFont = unsafe { mem::uninitialized() };
                let id = match *self {
                    % for font in system_fonts:
                        SystemFont::${to_camel_case(font)} => {
                            LookAndFeel_FontID::eFont_${to_camel_case(font.replace("-moz-", ""))}
                    }
                    % endfor
                };
                unsafe {
                    bindings::Gecko_nsFont_InitSystem(&mut system, id as i32,
                                            cx.style.get_font().gecko(),
                                            &*cx.device.pres_context)
                }
                let family = system.fontlist.mFontlist.iter().map(|font| {
                    use properties::longhands::font_family::computed_value::*;
                    FontFamily::FamilyName(FamilyName((&*font.mName).into()))
                }).collect::<Vec<_>>();
                let weight = unsafe {
                    longhands::font_weight::computed_value::T::from_gecko_weight(system.weight)
                };
                let ret = ComputedSystemFont {
                    font_family: longhands::font_family::computed_value::T(family),
                    font_size: Au(system.size),
                    font_weight: weight,
                    font_size_adjust: longhands::font_size_adjust::computed_value::T::from_gecko_adjust(system.sizeAdjust),
                    % for kwprop in kw_font_props:
                        ${kwprop}: longhands::${kwprop}::computed_value::T::from_gecko_keyword(system.style as u32),
                    % endfor
                    system_font: *self,
                };
                unsafe { bindings::Gecko_nsFont_Destroy(&mut system); }
                ret
            }

            fn from_computed_value(_: &ComputedSystemFont) -> Self {
                unreachable!()
            }
        }

        #[inline]
        /// Compute and cache a system font
        ///
        /// Must be called before attempting to compute a system font
        /// specified value
        pub fn resolve_system_font(system: SystemFont, context: &mut Context) {
            if context.style.cached_system_font.is_none() {
                let computed = system.to_computed_value(context);
                context.style.cached_system_font = Some(computed);
            }
            debug_assert!(system == context.style.cached_system_font.as_ref().unwrap().system_font)
        }

        #[derive(Clone, Debug)]
        pub struct ComputedSystemFont {
            % for name in SYSTEM_FONT_LONGHANDS:
                pub ${name}: longhands::${name}::computed_value::T,
            % endfor
            pub system_font: SystemFont,
        }

        impl SystemFont {
            pub fn parse(input: &mut Parser) -> Result<Self, ()> {
                Ok(match_ignore_ascii_case! { &*input.expect_ident()?,
                    % for font in system_fonts:
                        "${font}" => SystemFont::${to_camel_case(font)},
                    % endfor
                    _ => return Err(())
                })
            }
        }
    }
% else:
    pub mod system_font {
        use cssparser::Parser;
        use properties::longhands;
        use values::computed::Context;

        // We don't parse system fonts, but in the interest of not littering
        // a lot of code with `if product == gecko` conditionals, we have a
        // dummy system font module that does nothing

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        /// void enum for system font, can never exist
        pub enum SystemFont {}
        impl SystemFont {
            pub fn parse(_: &mut Parser) -> Result<Self, ()> {
                Err(())
            }
        }

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct ComputedSystemFont {
            % for name in "font_family font_size font_style font_stretch font_weight".split():
                pub ${name}: longhands::${name}::computed_value::T,
            % endfor
        }

        #[inline]
        /// Compute and cache a system font
        ///
        /// Must be called before attempting to compute a system font
        /// specified value
        pub fn resolve_system_font(_: SystemFont, _: &mut Context) {
            // do nothing, servo does not parse system fonts
        }
    }
% endif
