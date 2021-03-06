mod vfs;

use crate::prelude::*;

#[test]
fn then_test()
{
    let res = (then(keyword("hi"), keyword("bob"), left_right))(&ParserInput::new("hibob"));

    println!("{:#?}", res);

    assert_eq!(
        res,
        Ok(PRes {
            val :       (String::from("hi"), String::from("bob")),
            pos :       FilePos {
                line : 1, column : 5
            },
            remainder : "",
        },)
    );
}

#[test]
fn multi_element_parsers_test()
{
    let res = one_or_many(char_single('a'))(&ParserInput::new("aaaaabb123"));

    println!("{:#?}", res);

    assert_eq!(
        res,
        Ok(PRes {
            val :       vec!['a'; 5],
            pos :       FilePos {
                line : 1, column : 5
            },
            remainder : "bb123",
        },)
    );
}

#[test]
fn no_consume_test()
{
    let res = then(
        then(one_or_many(char_single('=')), no_consume(char_single('@')), left_right),
        then(char_single('@'), one_or_many(char_single('-')), left_right),
        left_right,
    )(&ParserInput::new("=======@--------"));

    println!("{:#?}", res);

    assert_eq!(
        res,
        Ok(PRes {
            val :       ((vec!['='; 7], '@'), ('@', vec!['-'; 8]),),
            pos :       FilePos {
                line :   1,
                column : 16,
            },
            remainder : "",
        },)
    );
}

#[test]
fn defer_test()
{
    let res =
        defer(|| then(defer(|| keyword("abc")), defer(|| keyword("123")), left_right))(&ParserInput::new("abc123"));

    println!("{:#?}", res);

    assert_eq!(
        res,
        Ok(PRes {
            val :       (String::from("abc"), String::from("123"),),
            pos :       FilePos {
                line : 1, column : 6
            },
            remainder : "",
        },)
    );
}

#[test]
fn test_or5()
{
    let res = or5(keyword("a"), keyword("b"), keyword("c"), keyword("d"), keyword("e"))(&ParserInput::new("c"));

    println!("{:#?}", res);
    assert_eq!(
        res,
        Ok(PRes {
            val :       Or5::C("c".to_string()),
            pos :       FilePos {
                line : 1, column : 1
            },
            remainder : "",
        },)
    )
}

#[test]
fn test_and()
{
    let res1 = and(keyword("a"), keyword("abc"), left_right)(&ParserInput::new("abcdef"));

    println!("Res1:\n{:#?}", res1);

    assert_eq!(
        res1,
        Ok(PRes {
            val :       ("a".to_string(), "abc".to_string()),
            pos :       FilePos {
                line : 1, column : 3
            },
            remainder : "def",
        },)
    );

    let res2 = and(keyword("abc"), keyword("a"), left_right)(&ParserInput::new("abcdef"));

    println!("Res2:\n{:#?}", res2);

    assert_eq!(
        res2,
        Ok(PRes {
            val :       ("abc".to_string(), "a".to_string()),
            pos :       FilePos {
                line : 1, column : 1
            },
            remainder : "bcdef",
        },)
    );
}
