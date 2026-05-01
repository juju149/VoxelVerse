use vv_registry::TagId;

pub(crate) fn tags_match(required: &[TagId], forbidden: &[TagId], provided: &[TagId]) -> bool {
    required.iter().all(|tag| provided.contains(tag))
        && forbidden.iter().all(|tag| !provided.contains(tag))
}
