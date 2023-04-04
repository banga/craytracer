#[cfg(test)]
mod tokenizer_tests {
    use craytracer::parser::{tokenize, ParserError, Token};

    #[test]
    fn simple() {
        assert_eq!(tokenize("").unwrap(), [Token::Eof]);
        assert_eq!(tokenize(" \r\t\n").unwrap(), [Token::Eof]);
        assert_eq!(
            tokenize("{}").unwrap(),
            [Token::LeftBrace, Token::RightBrace, Token::Eof]
        );
        assert_eq!(
            tokenize("[]").unwrap(),
            [Token::LeftBracket, Token::RightBracket, Token::Eof]
        );
        assert_eq!(
            tokenize("()").unwrap(),
            [Token::LeftParen, Token::RightParen, Token::Eof]
        );
        assert_eq!(tokenize("1").unwrap(), [Token::Number(1.0), Token::Eof]);
        assert_eq!(
            tokenize("'hello'").unwrap(),
            [Token::String("hello".to_string()), Token::Eof]
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(tokenize("1").unwrap(), [Token::Number(1.0), Token::Eof]);
        assert_eq!(tokenize("1.0").unwrap(), [Token::Number(1.0), Token::Eof]);
        assert_eq!(
            tokenize("1.0000").unwrap(),
            [Token::Number(1.0), Token::Eof]
        );
        assert_eq!(
            tokenize("001.0000").unwrap(),
            [Token::Number(1.0), Token::Eof]
        );

        assert_eq!(tokenize("-1").unwrap(), [Token::Number(-1.0), Token::Eof]);
        assert_eq!(tokenize("+1").unwrap(), [Token::Number(1.0), Token::Eof]);
        assert_eq!(tokenize("-1.1").unwrap(), [Token::Number(-1.1), Token::Eof]);
        assert_eq!(tokenize("+1.1").unwrap(), [Token::Number(1.1), Token::Eof]);

        // no expression support
        assert_eq!(
            tokenize("2+3").unwrap(),
            [Token::Number(2.0), Token::Number(3.0), Token::Eof]
        );
        assert_eq!(
            tokenize("4-5").unwrap(),
            [Token::Number(4.0), Token::Number(-5.0), Token::Eof]
        );

        // malformed numbers
        assert_eq!(
            tokenize("9.8.7").expect_err("Expected ParserError"),
            ParserError {
                message: "Unexpected character: '.'".to_string()
            }
        );
    }

    #[test]
    fn strings() {
        [
            // empty
            r#""""#,
            r#"''"#,
            // simple
            "'a'",
            r#""Once upon a midnight dreary""#,
            // mixed quotes
            r#"'"'"#,
            r#""'""#,
        ]
        .iter()
        .for_each(|s| {
            assert_eq!(
                tokenize(s).unwrap(),
                [Token::String(s[1..s.len() - 1].to_string()), Token::Eof],
            );
        });
    }

    #[test]
    fn identifiers() {
        [
            "simple",
            "snake_case",
            "camelCase",
            "SCREAMING_CASE",
            "agent007",
        ]
        .iter()
        .for_each(|s| {
            assert_eq!(
                tokenize(s).unwrap(),
                [Token::Identifier(s.to_string()), Token::Eof]
            );
        });
    }

    #[test]
    fn full() {
        assert_eq!(
            tokenize(
                r#"
{
    max_depth: 8,
    camera: ProjectionCamera {
        origin: Vector(0, 8, -10),
        fov: 5,
    },
    materials: {
        sky: Emissive {
            emittance: Color(0, 10, 60)
        }
    },
    shapes: {
        sky: Sphere {
            center: Vector(0, 0, 0),
            radius: 1000
        }
    },
    primitives: [
        Shape {
            shape: 'sky',
            material: 'sky'
        }
        Mesh {
            file: 'objs/triangle.obj'
        }
    ]
}
"#
            )
            .unwrap(),
            [
                Token::LeftBrace,
                Token::Identifier("max_depth".to_string()),
                Token::Colon,
                Token::Number(8.0),
                Token::Comma,
                Token::Identifier("camera".to_string()),
                Token::Colon,
                Token::Identifier("ProjectionCamera".to_string()),
                Token::LeftBrace,
                Token::Identifier("origin".to_string()),
                Token::Colon,
                Token::Identifier("Vector".to_string()),
                Token::LeftParen,
                Token::Number(0.0),
                Token::Comma,
                Token::Number(8.0),
                Token::Comma,
                Token::Number(-10.0),
                Token::RightParen,
                Token::Comma,
                Token::Identifier("fov".to_string()),
                Token::Colon,
                Token::Number(5.0),
                Token::Comma,
                Token::RightBrace,
                Token::Comma,
                Token::Identifier("materials".to_string()),
                Token::Colon,
                Token::LeftBrace,
                Token::Identifier("sky".to_string()),
                Token::Colon,
                Token::Identifier("Emissive".to_string()),
                Token::LeftBrace,
                Token::Identifier("emittance".to_string()),
                Token::Colon,
                Token::Identifier("Color".to_string()),
                Token::LeftParen,
                Token::Number(0.0),
                Token::Comma,
                Token::Number(10.0),
                Token::Comma,
                Token::Number(60.0),
                Token::RightParen,
                Token::RightBrace,
                Token::RightBrace,
                Token::Comma,
                Token::Identifier("shapes".to_string()),
                Token::Colon,
                Token::LeftBrace,
                Token::Identifier("sky".to_string()),
                Token::Colon,
                Token::Identifier("Sphere".to_string()),
                Token::LeftBrace,
                Token::Identifier("center".to_string()),
                Token::Colon,
                Token::Identifier("Vector".to_string()),
                Token::LeftParen,
                Token::Number(0.0),
                Token::Comma,
                Token::Number(0.0),
                Token::Comma,
                Token::Number(0.0),
                Token::RightParen,
                Token::Comma,
                Token::Identifier("radius".to_string()),
                Token::Colon,
                Token::Number(1000.0),
                Token::RightBrace,
                Token::RightBrace,
                Token::Comma,
                Token::Identifier("primitives".to_string()),
                Token::Colon,
                Token::LeftBracket,
                Token::Identifier("Shape".to_string()),
                Token::LeftBrace,
                Token::Identifier("shape".to_string()),
                Token::Colon,
                Token::String("sky".to_string()),
                Token::Comma,
                Token::Identifier("material".to_string()),
                Token::Colon,
                Token::String("sky".to_string()),
                Token::RightBrace,
                Token::RightBracket,
                Token::RightBrace,
                Token::Eof
            ]
        )
    }
}

#[cfg(test)]
mod parser_tests {
    use std::collections::HashMap;

    use craytracer::{
        color::Color,
        parser::{
            parse_raw_value, parse_scene, tokenize, ParserError, RawValue, RawValueArray,
            RawValueMap, TypedRawValueMap,
        },
        vector::Vector,
    };

    #[test]
    fn raw_value() {
        [
            ("1.23", RawValue::Number(1.23)),
            ("'hello'", RawValue::String("hello".to_string())),
            (
                "Vector(1, -2, 3.1)",
                RawValue::Vector(Vector(1.0, -2.0, 3.1)),
            ),
            (
                "Color(0, 0.5, 1)",
                RawValue::Color(Color {
                    r: 0.0,
                    g: 0.5,
                    b: 1.0,
                }),
            ),
            (
                "{}",
                RawValue::Map(RawValueMap {
                    map: HashMap::new(),
                }),
            ),
            (
                "{ x: 1, y: 'z' }",
                RawValue::Map(RawValueMap {
                    map: HashMap::from([
                        ("x".to_string(), RawValue::Number(1.0)),
                        ("y".to_string(), RawValue::String("z".to_string())),
                    ]),
                }),
            ),
            (
                "Sphere { center: Vector(0, 0, 0), radius: 1000 }",
                RawValue::TypedMap(TypedRawValueMap {
                    name: "Sphere".to_string(),
                    map: RawValueMap {
                        map: HashMap::from([
                            ("center".to_string(), RawValue::Vector(Vector::O)),
                            ("radius".to_string(), RawValue::Number(1000.0)),
                        ]),
                    },
                }),
            ),
            ("[]", RawValue::Array(RawValueArray { array: Vec::new() })),
            (
                "[1, 'foo', {}]",
                RawValue::Array(RawValueArray {
                    array: vec![
                        RawValue::Number(1.0),
                        RawValue::String("foo".to_string()),
                        RawValue::Map(RawValueMap {
                            map: HashMap::new(),
                        }),
                    ],
                }),
            ),
        ]
        .iter()
        .for_each(|(s, v)| {
            match parse_raw_value(&mut tokenize(s).unwrap().iter().peekable()) {
                Ok(result) => assert_eq!(result, *v, "Failed to parse {}", s),
                Err(e) => panic!("Failed to parse {}: {:?}", s, e),
            }
        });

        assert_eq!(
            parse_raw_value(&mut tokenize("x").unwrap().iter().peekable()),
            Err(ParserError {
                message: "Expected LeftBrace, got Eof".to_string()
            })
        );
    }

    #[test]
    fn raw_map() {
        [
            (
                "{}",
                RawValueMap {
                    map: HashMap::new(),
                },
            ),
            (
                "{ hello: 'world' }",
                RawValueMap {
                    map: HashMap::from([(
                        "hello".to_string(),
                        RawValue::String("world".to_string()),
                    )]),
                },
            ),
            (
                // Trailing comma
                "{ hello: 'world', }",
                RawValueMap {
                    map: HashMap::from([(
                        "hello".to_string(),
                        RawValue::String("world".to_string()),
                    )]),
                },
            ),
            (
                "{ x: 1, y: 'z', v: Vector(1,2,3), c: Color(1,0,0) }",
                RawValueMap {
                    map: HashMap::from([
                        ("x".to_string(), RawValue::Number(1.0)),
                        ("y".to_string(), RawValue::String("z".to_string())),
                        ("v".to_string(), RawValue::Vector(Vector(1.0, 2.0, 3.0))),
                        (
                            "c".to_string(),
                            RawValue::Color(Color {
                                r: 1.0,
                                g: 0.0,
                                b: 0.0,
                            }),
                        ),
                    ]),
                },
            ),
        ]
        .iter()
        .for_each(|(input, entries)| {
            assert_eq!(
                RawValueMap::from_tokens(&mut tokenize(input).unwrap().iter().peekable()).unwrap(),
                *entries
            )
        });

        assert_eq!(
            RawValueMap::from_tokens(&mut tokenize("{ x: 1, x: 2 }").unwrap().iter().peekable()),
            Err(ParserError {
                message: "Duplicate key x".to_string()
            })
        );
    }

    #[test]
    fn scene() {
        parse_scene(
            "
{
    max_depth: 5,
    camera: Projection {
        origin: Vector(0, 8, -10),
        target: Vector(1, 1.25, 12),
        up: Vector(0, 1, 0),
        focal_distance: 5,
        film_width: 896,
        film_height: 560,
        num_samples: 10
    },
    materials: {
        sky: Emissive {
            emittance: Color(0, 10, 60)
        },
        ground: Matte {
            reflectance: Color(1, 1, 1),
            sigma: 0
        },
        glass: Glass {
            reflectance: Color(1, 1, 1),
            transmittance: Color(1, 1, 1),
            eta: 1.75
        },
        light: Emissive {
            emittance: Color(255, 230, 20)
        }
    },
    shapes: {
        sky: Sphere {
            origin: Vector(0, 0, 0),
            radius: 1000
        },
        ground: Sphere {
            origin: Vector(0, -10000, 10),
            radius: 10000
        },
        glass: Sphere {
            origin: Vector(0, 1.5, 12.5),
            radius: 1.5
        },
        light: Sphere {
            origin: Vector(-3, 4, 13.5),
            radius: 0.5
        }
    },
    primitives: [
        { shape: 'sky', material: 'sky' },
        { shape: 'ground', material: 'ground' },
        { shape: 'glass', material: 'glass' },
        { shape: 'light', material: 'light' }
    ]
}
",
        )
        .unwrap();
    }
}
