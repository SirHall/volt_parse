#![feature(in_band_lifetimes, trait_alias, type_alias_impl_trait, clone_closures)]
use rayon::prelude::*;

#[derive(Clone)]
enum Dat
{
    None,
    String
    {
        s : String,
    },
    LR
    {
        l : Box<Dat>,
        r : Box<Dat>,
    },
    V
    {
        v : Box<Dat>,
    },
}

// Text position in the original file
#[derive(Default, Clone, Copy)]
struct FilePos
{
    line :   usize,
    column : usize,
}

impl FilePos
{
    fn new(line : usize, column : usize) -> Self
    {
        Self {
            line,
            column,
        }
    }
}

// Input text to be parsed
#[derive(Clone, Copy)]
struct InDat<'a>
{
    text : &'a str,
    pos :  FilePos,
}

#[derive(Clone)]
struct OutDat<'a>
{
    val :       Dat,
    pos :       FilePos,
    remainder : &'a str,
}

#[derive(Clone)]
struct FailDat {}

type Out<'a> = Result<OutDat<'a>, FailDat>;

// type Parser<'a> = impl Fn(&InDat<'a>) -> Out<'a>;
trait Parser<'a> = Fn(&InDat<'a>) -> Out<'a> + Clone;

trait Combiner<'a> = Fn(Out<'a>, Out<'a>) -> Out<'a> + Clone;
trait CombinerOk<'a> = Fn(OutDat<'a>, OutDat<'a>) -> Out<'a> + Clone;

fn gen_comb<'a>(a : Out<'a>, b : Out<'a>, comb : impl CombinerOk<'a>) -> Out<'a>
{
    match (a, b)
    {
        (Ok(a), Ok(b)) => comb(a, b),
        (Err(a), Ok(_)) => Err(a),
        (Ok(_), Err(b)) => Err(b),
        (_, _) => Err(FailDat {}),
    }
}

fn lr_comb<'a>(a : Out<'a>, b : Out<'a>) -> Out<'a>
{
    gen_comb(a, b, |c1 : OutDat<'a>, c2 : OutDat<'a>| {
        Ok(OutDat {
            val : Dat::LR {
                l : c1.val.into(),
                r : c2.val.into(),
            },
            ..c2
        })
    })
}

fn l_comb<'a>(a : Out<'a>, b : Out<'a>) -> Out<'a>
{
    gen_comb(a, b, |c1 : OutDat<'a>, c2 : OutDat<'a>| {
        Ok(OutDat {
            val : Dat::V {
                v : c1.val.into()
            },
            ..c2
        })
    })
}

fn r_comb<'a>(a : Out<'a>, b : Out<'a>) -> Out<'a>
{
    gen_comb(a, b, |c1 : OutDat<'a>, c2 : OutDat<'a>| {
        Ok(OutDat {
            val : Dat::V {
                v : c2.val.into()
            },
            ..c2
        })
    })
}

impl OutDat<'a>
{
    fn to_in(&self) -> InDat<'a>
    {
        InDat {
            pos :  self.pos,
            text : self.remainder,
        }
    }
}

fn then<'a>(a : impl Parser<'a>, b : impl Parser<'a>, comb : impl Combiner<'a>) -> impl Parser<'a>
{
    move |ind : &InDat<'a>| -> Out<'a> {
        match a(ind)
        {
            Ok(a_succ) =>
            {
                let b_res = b(&a_succ.to_in());
                comb(Ok(a_succ), b_res)
            },
            Err(a_fail) => Err(a_fail),
        }
    }
}

fn mod_dat<'a>(p : impl Parser<'a>, f : impl Fn(OutDat<'a>) -> OutDat<'a> + Clone) -> impl Parser<'a>
{
    move |ind : &InDat<'a>| -> Out<'a> {
        let d = p(ind);
        match d
        {
            Ok(suc) => Ok(f(suc)),
            Err(fail) => Err(fail),
        }
    }
}

fn mod_val<'a>(p : impl Parser<'a>, f : impl Fn(Dat) -> Dat + Clone) -> impl Parser<'a>
{
    mod_dat(p, move |od : OutDat<'a>| -> OutDat<'a> {
        OutDat {
            val :       f(od.val),
            pos :       od.pos,
            remainder : od.remainder,
        }
    })
}

fn replace_val<'a>(p : impl Parser<'a>, v : Dat) -> impl Parser<'a>
{
    mod_dat(p, move |od : OutDat<'a>| -> OutDat<'a> {
        OutDat {
            val :       v.clone(),
            pos :       od.pos,
            remainder : od.remainder,
        }
    })
}

fn succeed_if<'a>(p : impl Parser<'a>, f : impl Fn(&OutDat<'a>) -> bool + Clone) -> impl Parser<'a>
{
    move |ind : &InDat<'a>| -> Out<'a> {
        p(ind).and_then(|r| {
            if f(&r)
            {
                Ok(r)
            }
            else
            {
                Err(FailDat {})
            }
        })
    }
}

fn fail_if<'a>(p : impl Parser<'a>, f : impl Fn(&OutDat<'a>) -> bool + Clone) -> impl Parser<'a>
{
    succeed_if(p, move |r| !f(r))
}

fn consume_all<'a>(p : impl Parser<'a>) -> impl Parser<'a> { succeed_if(p, |r| r.remainder.is_empty()) }

// TODO: Replace/Modify fail message

// TODO: This may not work, the lifetimes probably aren't right
fn always<'a>(v : &'a str) -> impl Parser<'a>
{
    move |ind : &InDat<'a>| -> Out<'a> {
        Ok(OutDat {
            val :       Dat::String {
                s : String::from(v)
            },
            pos :       Default::default(),
            remainder : Default::default(),
        })
    }
}

fn not<'a>(p : impl Parser<'a>) -> impl Parser<'a>
{
    move |ind : &InDat<'a>| -> Out<'a> {
        match p(ind)
        {
            Ok(v) => Err(FailDat {}),
            Err(v) => Ok(OutDat {
                val :       Dat::None,
                pos :       ind.pos,
                remainder : ind.text,
            }),
        }
    }
}

fn or<'a>(a : impl Parser<'a>, b : impl Parser<'a>) -> impl Parser<'a>
{
    move |ind : &InDat<'a>| -> Out<'a> {
        match a(ind)
        {
            Ok(a_succ) => Ok(a_succ),
            Err(a_fail) => b(ind),
        }
    }
}

fn either_or<'a>(a : impl Parser<'a>, b : impl Parser<'a>, comb : impl Combiner<'a>) -> impl Parser<'a>
{
    or(then(a.clone(), b.clone(), comb), or(a, b))
}

fn chain<'a>(ps : Vec<impl Parser<'a>>, comb : impl Combiner<'a>) -> impl Parser<'a>
{
    move |ind : &InDat<'a>| -> Out<'a> {
        if ps.len() == 0
        {
            Err(FailDat {})
        }
        else
        {
            let mut out = ps[0](ind);

            for i in 1..ps.len()
            {
                match out.as_ref()
                {
                    Ok(succ) => out = comb(out.to_owned(), ps[i](&succ.to_in())),
                    Err(fail) => return Err(FailDat {}),
                };
            }

            out
        }
    }
}

fn chain_select<'a>(ps : Vec<impl Parser<'a>>, index : usize) -> impl Parser<'a>
{
    move |ind : &InDat<'a>| -> Out<'a> {
        if ps.len() == 0
        {
            Err(FailDat {})
        }
        else
        {
            // Ensure that all succeed, but only return the value of the selected parser

            let mut selected : Option<OutDat> = None;

            let mut all = ps[0](ind);

            for i in 1..ps.len()
            {
                match all.as_ref()
                {
                    Ok(succ) =>
                    {
                        if i == index
                        {
                            selected = all.clone().ok();
                        }
                        all = r_comb(all.to_owned(), ps[i](&succ.to_in()));
                    },
                    Err(_) => return Err(FailDat {}),
                };
            }

            match (selected, all)
            {
                (Some(s), Ok(a)) => Ok(OutDat {
                    val : s.val,
                    ..a
                }),
                (_, _) => Err(FailDat {}),
            }
        }
    }
}

fn or_chain<'a>(ps : Vec<impl Parser<'a>>) -> impl Parser<'a>
{
    move |ind : &InDat<'a>| -> Out<'a> {
        for i in 0..ps.len()
        {
            if let Ok(succ) = ps[i](ind)
            {
                return Ok(succ);
            }
        }

        return Err(FailDat {});
    }
}

fn main() {}
