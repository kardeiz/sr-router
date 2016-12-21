extern crate hyper;

#[macro_use]
extern crate quick_error;

pub mod err {
    quick_error! {
        #[derive(Debug)]
        pub enum Error {
            Other(err: Box<::std::error::Error + Send + Sync>) {
                from(e: &'static str) -> (e.into())
                description(err.description())
                display("{}", err)
            }          
        }
    }
    pub type Result<T> = ::std::result::Result<T, Error>;
}

pub mod tree;

use hyper::server::{Handler, Response, Request as HyperRequest};
use hyper::method::Method;
use hyper::uri::RequestUri;


use std::ops::{Deref, DerefMut};
use std::convert::From;

pub struct RequestExtensions {
    path_delims: Option<(usize, Option<usize>)>,
    capture_delims: Option<Vec<(usize, usize)>>
}

pub struct Request<'a, 'b: 'a> {
    inner: HyperRequest<'a, 'b>,
    extensions: RequestExtensions
}


impl<'a, 'b: 'a> Deref for Request<'a, 'b> {
    type Target = HyperRequest<'a, 'b>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, 'b: 'a> DerefMut for Request<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, 'b: 'a> From<HyperRequest<'a, 'b>> for Request<'a, 'b> {

    fn from(x: HyperRequest<'a, 'b>) -> Self {        
        let path_delims = match x.uri {            
            ::hyper::uri::RequestUri::AbsolutePath(ref s) => {                
                if let Some(pos) = s.find('?') {
                    Some((pos, Some(pos+1)))
                } else {
                    Some((s.len(), None))
                }
            },
            _ => None
        };

        let extensions = RequestExtensions {
            path_delims: path_delims,
            capture_delims: None
        };

        Request {
            inner: x,
            extensions: extensions
        }
    }

}

impl<'a, 'b: 'a> Request<'a, 'b> {

    pub fn path(&self) -> &str {
        match self.inner.uri {
            RequestUri::AbsolutePath(ref s) => {
                let pos = self.extensions.path_delims
                    .map(|x| x.0)
                    .expect("Path end delim must be set");
                &s[..pos]
            },
            RequestUri::AbsoluteUri(ref url) => url.path(),
            _ => panic!("Unexpected request URI")
        }
    }

    pub fn query(&self) -> Option<&str> {
        match self.inner.uri {
            RequestUri::AbsolutePath(ref s) => {
                self.extensions.path_delims
                    .and_then(|x| x.1)
                    .map(|pos| &s[pos..] )
            },
            RequestUri::AbsoluteUri(ref url) => url.query(),
            _ => panic!("Unexpected request URI")
        }
    }

    pub fn captures(&self) -> Vec<&str> {
        let path = self.path();
        self.extensions.capture_delims
            .iter()
            .flat_map(|x| x)
            .map(|&(d0, d1)| &path[d0..d1] )
            .collect()
    }

}

pub trait InnerHandler: Send + Sync {
    fn handle<'a, 'b>(&'a self, Request<'a, 'b>, Response<'a>);
}

impl<F> InnerHandler for F where F: Fn(Request, Response) + Sync + Send {
    fn handle<'a, 'b>(&'a self, 
        req: Request<'a, 'b>, 
        res: Response<'a>) {
        self(req, res)
    }
}

impl Router {
     pub fn build() -> RouterBuilder { RouterBuilder::default() }
}

impl RouterBuilder {
    pub fn not_found<H: InnerHandler + 'static>(mut self, handler: H) -> Self {
        self.not_found = Some(Box::new(handler));
        self
    }
}

macro_rules! impls {
    ($([
        $prefix_tree:ident,
        $he:pat, 
        $add:ident]),+) => {
        
        pub struct Router {
            $(
                $prefix_tree: Option<tree::Root<Box<InnerHandler>>>,
            )+
            not_found: Box<InnerHandler>
        }

        unsafe impl Send for Router {}
        unsafe impl Sync for Router {}

        impl Handler for Router {
            fn handle<'a, 'k>(&'a self, req: HyperRequest<'a, 'k>, res: Response<'a>) {
                let mut req = Request::from(req);
                match req.method {
                    $(
                        $he => {
                            if let Some((handler, capture_delims)) = self.$prefix_tree
                                .as_ref()
                                .and_then(|x| x.check(req.path())) {                                
                                req.extensions.capture_delims = capture_delims;
                                handler.handle(req, res);
                                return;
                            }                                
                        },
                    )+
                    _ => { }
                }
                self.not_found.handle(req, res)
            }
        }

        #[derive(Default)]
        pub struct RouterBuilder {
            $(
                $prefix_tree: Option<tree::Root<Box<InnerHandler>>>,
            )+
            not_found: Option<Box<InnerHandler>>
        }

        impl RouterBuilder {
            $(
                pub fn $add<B>(mut self, re: &str, handler: B) -> Self
                    where B: InnerHandler + 'static {
                    
                    let mut root = self.$prefix_tree
                        .take()
                        .unwrap_or_else(tree::Root::new);

                    root.insert(re, Box::new(handler));

                    self.$prefix_tree = Some(root);

                    self
                }
            )+

            pub fn finish(self) -> ::err::Result<Router> {                

                let out = Router {
                    $(
                        $prefix_tree: self.$prefix_tree,
                    )+
                    not_found: self.not_found.ok_or("Must include not found")?
                };
                Ok(out)
            }
        }
    }
}

impls!{
    [get_tree, Method::Get, add_get],
    [post_tree, Method::Post, add_post],
    [put_tree, Method::Put, add_put],
    [patch_tree, Method::Patch, add_patch],
    [delete_tree, Method::Delete, add_delete],
    [head_tree, Method::Head, add_head]
}





#[cfg(test)]
mod tests {

    use super::*;

}
