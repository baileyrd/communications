//! CSR/1: a deliberately local typed-record experiment, not a standard.
use crate::{
    Assertion, Contract, Entity, Evidence, Method, Object, Provenance, Relation, Times, Uncertainty,
};
use std::collections::{BTreeMap, BTreeSet};

const MAGIC: &[u8; 4] = b"CSR\x01";
pub(super) const MAX_ITEMS: usize = 4096;
pub(super) const MAX_STRING: usize = 65_536;

struct Writer(Vec<u8>);
impl Writer {
    fn byte(&mut self, value: u8) {
        self.0.push(value);
    }
    fn count(&mut self, count: usize) -> Result<(), String> {
        if count > MAX_ITEMS {
            return Err("record collection exceeds 4096".into());
        }
        self.0.extend_from_slice(&(count as u32).to_le_bytes());
        Ok(())
    }
    fn text(&mut self, text: &str) -> Result<(), String> {
        if text.len() > MAX_STRING {
            return Err("record string exceeds 65536 bytes".into());
        }
        self.0.extend_from_slice(&(text.len() as u32).to_le_bytes());
        self.0.extend_from_slice(text.as_bytes());
        super::bound(self.0.len())
    }
    fn int(&mut self, value: i64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }
}

pub(super) fn encode(c: &Contract) -> Result<Vec<u8>, String> {
    let mut w = Writer(MAGIC.to_vec());
    w.text(&c.version)?;
    w.count(c.entities.len())?;
    for (id, e) in &c.entities {
        w.byte(0x10);
        w.text(id)?;
        w.text(&e.kind)?;
    }
    w.count(c.relations.len())?;
    for (id, r) in &c.relations {
        w.byte(0x11);
        w.text(id)?;
        w.text(&r.subject_kind)?;
        w.text(&r.object_kind)?;
    }
    w.count(c.evidence.len())?;
    for (id, e) in &c.evidence {
        w.byte(0x12);
        w.text(id)?;
        // Contract validation has already required exactly 64 lowercase hex digits.
        for pair in e.sha256.as_bytes().as_chunks::<2>().0 {
            fn nibble(b: u8) -> u8 {
                if b <= b'9' {
                    b - b'0'
                } else {
                    b - b'a' + 10
                }
            }
            w.byte((nibble(pair[0]) << 4) | nibble(pair[1]));
        }
        w.text(&e.locator)?;
    }
    let mut assertions: Vec<_> = c.assertions.iter().collect();
    assertions.sort_by(|a, b| a.id.cmp(&b.id));
    w.count(assertions.len())?;
    for a in assertions {
        w.byte(0x13);
        w.text(&a.id)?;
        w.text(&a.subject)?;
        w.text(&a.predicate)?;
        match &a.object {
            Object::Entity(v) => {
                w.byte(0x20);
                w.text(v)?;
            }
            Object::Boolean(v) => {
                w.byte(0x21);
                w.byte(u8::from(*v));
            }
            Object::Text(v) => {
                w.byte(0x22);
                w.text(v)?;
            }
            Object::Integer(v) => {
                w.byte(0x23);
                w.int(*v);
            }
        }
        match a.uncertainty {
            Uncertainty::Unknown => w.byte(0),
            Uncertainty::Confidence { permille } => {
                w.byte(1);
                w.0.extend_from_slice(&permille.to_le_bytes());
            }
        }
        w.text(&a.provenance.asserted_by)?;
        w.byte(match a.provenance.method {
            Method::Observed => 0,
            Method::Reported => 1,
            Method::Inferred => 2,
        });
        w.count(a.provenance.evidence.len())?;
        for id in &a.provenance.evidence {
            w.text(id)?;
        }
        match a.times.event_ms {
            None => w.byte(0),
            Some(value) => {
                w.byte(1);
                w.int(value);
            }
        }
        w.int(a.times.record_ms);
        w.int(a.times.interpretation_ms);
    }
    Ok(w.0)
}

struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let value = self.0.get(..n).ok_or("truncated record")?;
        self.0 = &self.0[n..];
        Ok(value)
    }
    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], String> {
        self.take(N)?
            .try_into()
            .map_err(|_| "truncated scalar".into())
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.fixed::<1>()?[0])
    }
    fn expect(&mut self, value: u8) -> Result<(), String> {
        if self.byte()? != value {
            return Err("unknown/out-of-order record tag".into());
        }
        Ok(())
    }
    fn len(&mut self, limit: usize) -> Result<usize, String> {
        let n = u32::from_le_bytes(self.fixed()?) as usize;
        if n > limit {
            return Err("record length exceeds limit".into());
        }
        Ok(n)
    }
    fn count(&mut self) -> Result<usize, String> {
        self.len(MAX_ITEMS)
    }
    fn text(&mut self) -> Result<String, String> {
        let n = self.len(MAX_STRING)?;
        String::from_utf8(self.take(n)?.to_vec()).map_err(|e| e.to_string())
    }
    fn int(&mut self) -> Result<i64, String> {
        Ok(i64::from_le_bytes(self.fixed()?))
    }
}

fn insert<T>(map: &mut BTreeMap<String, T>, id: String, value: T) -> Result<(), String> {
    if map.insert(id, value).is_some() {
        return Err("duplicate record identifier".into());
    }
    Ok(())
}

pub(super) fn decode(bytes: &[u8]) -> Result<Contract, String> {
    let mut r = Reader(bytes);
    if r.take(4)? != MAGIC {
        return Err("unknown CSR framing/version".into());
    }
    let version = r.text()?;
    let mut entities = BTreeMap::new();
    for _ in 0..r.count()? {
        r.expect(0x10)?;
        let id = r.text()?;
        insert(&mut entities, id, Entity { kind: r.text()? })?;
    }
    let mut relations = BTreeMap::new();
    for _ in 0..r.count()? {
        r.expect(0x11)?;
        let id = r.text()?;
        insert(
            &mut relations,
            id,
            Relation {
                subject_kind: r.text()?,
                object_kind: r.text()?,
            },
        )?;
    }
    let mut evidence = BTreeMap::new();
    for _ in 0..r.count()? {
        r.expect(0x12)?;
        let id = r.text()?;
        let sha256 = r.take(32)?.iter().map(|b| format!("{b:02x}")).collect();
        insert(
            &mut evidence,
            id,
            Evidence {
                sha256,
                locator: r.text()?,
            },
        )?;
    }
    let mut assertions = Vec::new();
    for _ in 0..r.count()? {
        r.expect(0x13)?;
        let id = r.text()?;
        let subject = r.text()?;
        let predicate = r.text()?;
        let object = match r.byte()? {
            0x20 => Object::Entity(r.text()?),
            0x21 => Object::Boolean(match r.byte()? {
                0 => false,
                1 => true,
                _ => return Err("invalid boolean".into()),
            }),
            0x22 => Object::Text(r.text()?),
            0x23 => Object::Integer(r.int()?),
            _ => return Err("unknown object tag".into()),
        };
        let uncertainty = match r.byte()? {
            0 => Uncertainty::Unknown,
            1 => Uncertainty::Confidence {
                permille: u16::from_le_bytes(r.fixed()?),
            },
            _ => return Err("unknown uncertainty tag".into()),
        };
        let asserted_by = r.text()?;
        let method = match r.byte()? {
            0 => Method::Observed,
            1 => Method::Reported,
            2 => Method::Inferred,
            _ => return Err("unknown method tag".into()),
        };
        let mut refs = BTreeSet::new();
        for _ in 0..r.count()? {
            if !refs.insert(r.text()?) {
                return Err("duplicate evidence reference".into());
            }
        }
        let event_ms = match r.byte()? {
            0 => None,
            1 => Some(r.int()?),
            _ => return Err("unknown event-time tag".into()),
        };
        assertions.push(Assertion {
            id,
            subject,
            predicate,
            object,
            uncertainty,
            provenance: Provenance {
                asserted_by,
                method,
                evidence: refs,
            },
            times: Times {
                event_ms,
                record_ms: r.int()?,
                interpretation_ms: r.int()?,
            },
        });
    }
    if !r.0.is_empty() {
        return Err("trailing record bytes".into());
    }
    let c = Contract {
        version,
        entities,
        relations,
        evidence,
        assertions,
    };
    c.validate()?;
    Ok(c)
}
