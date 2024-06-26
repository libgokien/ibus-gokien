use crate::c::{g_list_free, GList};

#[derive(Debug)]
pub struct EngineIter {
    origin: *mut GList,
    p: *mut GList,
}

impl EngineIter {
    pub fn new(origin: *mut GList) -> Self {
        Self { origin, p: origin }
    }
}

impl Drop for EngineIter {
    fn drop(&mut self) {
        unsafe {
            g_list_free(self.origin);
        }
    }
}

impl Iterator for EngineIter {
    type Item = super::EngineDesc;

    fn next(&mut self) -> Option<Self::Item> {
        if self.p.is_null() {
            return None;
        }
        unsafe {
            let item = super::EngineDesc((*(self.p)).data.cast());
            self.p = (*(self.p)).next;
            Some(item)
        }
    }
}
