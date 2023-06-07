use std::collections::{btree_map::Entry, BTreeMap};

use crate::{state::StateMap, ASGIResponse, ASGIType};
use bytes::{BufMut, Bytes, BytesMut};
use http::response::Builder;
type HeaderInternal<'a> = BTreeMap<String, Vec<&'a Bytes>>;

/// Insets headers from an ASGI response event (application->server), coalescing duplicate values
/// into a vec to later join if required (HTTP)
#[inline(always)]
fn inset_headers<'a>(
    mut headers: HeaderInternal<'a>,
    asgi: &'a ASGIResponse,
) -> HeaderInternal<'a> {
    if !asgi.headers.is_empty() {
        for (k, bt) in asgi.headers.iter() {
            match headers.entry(k.to_string()) {
                Entry::Vacant(e) => {
                    e.insert(vec![bt]);
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().push(bt);
                }
            };
        }
    }
    headers
}

/// Coalesces `Builder` and body `Bytes` from a series of ASGI response events (application->server)
/// State is a [`BTreeSet`] (ASGIResponse), internalized as `StateMap`.
/// Builder is a [`Builder`] (hyper)
/// Is HTTP indicates whether the headers are to be joined and normalized
pub fn coslesce_from_state<'a>(
    state: &'a StateMap,
    mut builder: Builder,
    _ishttp: bool, // TODO: handle non http vec
) -> (Builder, Bytes) {
    let mut body_pieces: Vec<BytesMut> = vec![];
    let mut headers: HeaderInternal = BTreeMap::default();

    for asgi in state.0.iter() {
        match asgi._type {
            ASGIType::HTTPResponseBody => {
                if asgi.body.is_some() {
                    let mu: &'a [u8] = asgi.body.as_ref().unwrap().as_ref();
                    body_pieces.push(BytesMut::from(mu));
                }
                headers = inset_headers(headers, asgi);
            }
            ASGIType::HTTPResponseStart => {
                if asgi.status.is_some() {
                    builder = builder.status(asgi.status.unwrap())
                }
                headers = inset_headers(headers, asgi);
            }
            ASGIType::HTTPResponseTrailers => {}
            _ => {}
        }
    }
    let mut body = BytesMut::new();

    if body_pieces.len() > 0 {
        for (_, bts) in body_pieces.iter().enumerate() {
            body.put((*bts).as_ref());
        }
    }

    if headers.len() > 0 {
        for (key, head) in headers.iter() {
            let header = head
                .iter()
                .map(|by| {
                    let b = by.as_ref();
                    let r = std::str::from_utf8(b).expect("Headers must be UTF-8 Compliant");
                    r
                })
                .collect::<Vec<&str>>()
                .join(",");
            builder = builder.header(key, header);
        }
    }

    (builder, body.into())
}
