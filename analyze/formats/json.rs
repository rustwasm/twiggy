// A couple methods are dead, but removing them would make the API oddly
// imbalanced and we might want to use them in some future analysis.
#![allow(dead_code)]

use std::io;

pub trait JsonPrimitive {
    fn json_primitive(&self, w: &mut io::Write) -> io::Result<()>;
}

impl<'a> JsonPrimitive for &'a str {
    fn json_primitive(&self, w: &mut io::Write) -> io::Result<()> {
        write!(w, "\"")?;
        for c in self.chars() {
            match c {
                '"' => write!(w, "\\\"")?,
                '\\' => write!(w, "\\")?,
                '\n' => writeln!(w,)?,
                c => write!(w, "{}", c)?,
            }
        }
        write!(w, "\"")
    }
}

impl JsonPrimitive for f64 {
    fn json_primitive(&self, w: &mut io::Write) -> io::Result<()> {
        write!(w, "{}", self)
    }
}

impl JsonPrimitive for u32 {
    fn json_primitive(&self, w: &mut io::Write) -> io::Result<()> {
        write!(w, "{}", self)
    }
}

pub fn array(w: &mut io::Write) -> io::Result<Array> {
    write!(w, "[")?;
    Ok(Array {
        w,
        need_comma: false,
    })
}

pub fn object(w: &mut io::Write) -> io::Result<Object> {
    write!(w, "{{")?;
    Ok(Object {
        w,
        need_comma: false,
    })
}

pub struct Array<'a> {
    w: &'a mut io::Write,
    need_comma: bool,
}

impl<'a> Drop for Array<'a> {
    fn drop(&mut self) {
        let _ = write!(self.w, "]");
    }
}

impl<'a> Array<'a> {
    fn comma(&mut self) -> io::Result<()> {
        if self.need_comma {
            write!(self.w, ",")?;
        }
        self.need_comma = true;
        Ok(())
    }

    pub fn object(&mut self) -> io::Result<Object> {
        self.comma()?;
        object(&mut *self.w)
    }

    pub fn array(&mut self) -> io::Result<Array> {
        self.comma()?;
        array(&mut *self.w)
    }

    pub fn elem<P>(&mut self, elem: P) -> io::Result<()>
    where
        P: JsonPrimitive,
    {
        self.comma()?;
        elem.json_primitive(self.w)
    }
}

pub struct Object<'a> {
    w: &'a mut io::Write,
    need_comma: bool,
}

impl<'a> Drop for Object<'a> {
    fn drop(&mut self) {
        let _ = write!(self.w, "}}");
    }
}

impl<'a> Object<'a> {
    fn comma_and_name<S>(&mut self, name: S) -> io::Result<()>
    where
        S: AsRef<str>,
    {
        if self.need_comma {
            write!(self.w, ",")?;
        }
        self.need_comma = true;
        name.as_ref().json_primitive(self.w)?;
        write!(self.w, ":")
    }

    pub fn object<S>(&mut self, name: S) -> io::Result<Object>
    where
        S: AsRef<str>,
    {
        self.comma_and_name(name)?;
        object(&mut *self.w)
    }

    pub fn array<S>(&mut self, name: S) -> io::Result<Array>
    where
        S: AsRef<str>,
    {
        self.comma_and_name(name)?;
        array(&mut *self.w)
    }

    pub fn field<S, P>(&mut self, name: S, val: P) -> io::Result<()>
    where
        S: AsRef<str>,
        P: JsonPrimitive,
    {
        self.comma_and_name(name)?;
        val.json_primitive(self.w)
    }
}
