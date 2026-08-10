//! Schema catalog: durable field identity for YYDB (host runtime, not `vos-ast`).
//!
//! Normative design: `vos-language/docs/field-identity.md` and `docs/ddl.md`.
//!
//! This catalog is persisted inside the authoritative `.yydb` file.
//! It maps VOS source names onto stable `FieldId` + `VirtualFieldIndex` slots so
//! rename/reorder can preserve IR bindings without `renamed_from` aliases.

use std::collections::{BTreeMap, BTreeSet};

use vos::ast::{BuiltinType, Document, Field, FieldAttribute, Item, TypeExpr};
use yydb_types::{Error, Result};

use crate::macros::{
    blake3_hex, lower_macro_source, rewrite_field_name, semantic_hash_of, FieldRef, MacroEntry,
    MacroId, MacroIr,
};

/// Stable type identity (table / class) inside one database catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeId(pub u64);

/// Stable field identity across rename / reorder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldId(pub u64);

/// Virtual field slot index — assigned once, never reused.
pub type VirtualFieldIndex = u32;

/// Catalog publish counters (see vos-language field-identity doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Revisions {
    /// Canonical DDL / catalog publish generation.
    pub ddl: u64,
    /// Observable type / constraint semantics generation.
    pub semantic: u64,
    /// Physical row encoding generation (layout descriptor).
    pub layout_epoch: u64,
}

/// Where a live field currently sits in storage (placeholder until pager rows).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PhysicalLocation {
    /// Not yet bound to a page/column encoding.
    Unbound,
}

/// One live field in a type’s virtual slot map.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDesc {
    /// Durable identity.
    pub field_id: FieldId,
    /// Current source / display name.
    pub current_name: String,
    /// Source declaration order (reorder changes only this).
    pub source_order: u32,
    /// VOS type expression.
    pub ty: TypeExpr,
    /// Primary / unique attributes.
    pub attrs: Vec<FieldAttribute>,
    /// Physical binding (layout layer).
    pub location: PhysicalLocation,
}

/// One virtual slot: live field or tombstone.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Slot {
    /// Active field.
    Live(FieldDesc),
    /// Dropped field — index must never be reused.
    Tombstone {
        /// Former field id.
        field_id: FieldId,
        /// Name at drop time (audit only; not an active alias).
        former_name: String,
    },
}

impl Slot {
    /// Live field descriptor, if any.
    pub fn live(&self) -> Option<&FieldDesc> {
        match self {
            Self::Live(desc) => Some(desc),
            Self::Tombstone { .. } => None,
        }
    }

    /// Mutable live field descriptor, if any.
    pub fn live_mut(&mut self) -> Option<&mut FieldDesc> {
        match self {
            Self::Live(desc) => Some(desc),
            Self::Tombstone { .. } => None,
        }
    }
}

/// Kind of named type in the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TypeKind {
    /// Persistence `table`.
    Table,
    /// Non-persistent `class`.
    Class,
}

/// One table or class and its virtual field slots.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeEntry {
    /// Stable type id.
    pub type_id: TypeId,
    /// Current type name.
    pub name: String,
    /// Table vs class.
    pub kind: TypeKind,
    /// Virtual slots (including tombstones).
    pub slots: Vec<Slot>,
    /// Current-name → slot for live fields only.
    name_to_slot: BTreeMap<String, VirtualFieldIndex>,
}

impl TypeEntry {
    /// Resolve a current field name to `(FieldId, VirtualFieldIndex)`.
    pub fn resolve(&self, field_name: &str) -> Result<(FieldId, VirtualFieldIndex)> {
        let slot = *self
            .name_to_slot
            .get(field_name)
            .ok_or_else(|| Error::Schema {
                message: format!(
                    "VOS-DDL-UNKNOWN-FIELD: `{}.{}` is not a live field",
                    self.name, field_name
                ),
                span: None,
                hint: None,
            })?;
        let field_id = self.slots[slot as usize]
            .live()
            .ok_or_else(|| Error::Corrupt("name map points at tombstone"))?
            .field_id;
        Ok((field_id, slot))
    }

    /// Current name for a virtual slot (`None` if tombstone / OOB).
    pub fn name_at(&self, slot: VirtualFieldIndex) -> Option<&str> {
        self.slots
            .get(slot as usize)
            .and_then(|s| s.live().map(|d| d.current_name.as_str()))
    }

    /// Live fields in `source_order`.
    pub fn live_fields_in_source_order(&self) -> Vec<&FieldDesc> {
        let mut live: Vec<&FieldDesc> = self.slots.iter().filter_map(Slot::live).collect();
        live.sort_by_key(|d| d.source_order);
        live
    }

    fn rebuild_name_index(&mut self) {
        self.name_to_slot.clear();
        for (idx, slot) in self.slots.iter().enumerate() {
            if let Some(desc) = slot.live() {
                self.name_to_slot
                    .insert(desc.current_name.clone(), idx as VirtualFieldIndex);
            }
        }
    }
}

/// In-memory schema catalog for one database.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaCatalog {
    /// Revision counters.
    pub revisions: Revisions,
    types: BTreeMap<String, TypeEntry>,
    macros: BTreeMap<String, MacroEntry>,
    next_type_id: u64,
    next_field_id: u64,
    next_macro_id: u64,
}

impl SchemaCatalog {
    /// Empty catalog at revision 0.
    pub fn new() -> Self {
        Self {
            revisions: Revisions::default(),
            types: BTreeMap::new(),
            macros: BTreeMap::new(),
            next_type_id: 1,
            next_field_id: 1,
            next_macro_id: 1,
        }
    }

    /// Build a catalog from a parsed VOS [`Document`] (initial slots = source order).
    pub fn from_document(document: &Document) -> Result<Self> {
        let mut catalog = Self::new();
        for item in &document.items {
            match item {
                Item::Table(table) => {
                    catalog.add_type(TypeKind::Table, &table.name, &table.fields)?;
                }
                Item::Class(class) => {
                    catalog.add_type(TypeKind::Class, &class.name, &class.fields)?;
                }
                _ => {}
            }
        }
        // Initial publish.
        catalog.revisions.ddl = 1;
        catalog.revisions.semantic = 1;
        Ok(catalog)
    }

    /// Look up a type by current name.
    pub fn type_entry(&self, type_name: &str) -> Result<&TypeEntry> {
        self.types.get(type_name).ok_or_else(|| Error::Schema {
            message: format!("VOS-DDL-UNKNOWN-FIELD: unknown type `{type_name}`"),
            span: None,
            hint: None,
        })
    }

    /// Mutable type entry.
    pub fn type_entry_mut(&mut self, type_name: &str) -> Result<&mut TypeEntry> {
        self.types.get_mut(type_name).ok_or_else(|| Error::Schema {
            message: format!("VOS-DDL-UNKNOWN-FIELD: unknown type `{type_name}`"),
            span: None,
            hint: None,
        })
    }

    /// Iterate type entries.
    pub fn types(&self) -> impl Iterator<Item = &TypeEntry> {
        self.types.values()
    }

    /// `Database.rename_field(Type.old, new)` — identity preserved; no active alias.
    pub fn rename_field(
        &mut self,
        type_name: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<(FieldId, VirtualFieldIndex)> {
        if old_name == new_name {
            return self.type_entry(type_name)?.resolve(old_name);
        }
        if new_name.is_empty() {
            return Err(Error::Schema {
                message: "VOS-DDL-NAME-CONFLICT: field name must be non-empty".into(),
                span: None,
                hint: None,
            });
        }

        // Conflict check before borrow mutably for update.
        {
            let entry = self.type_entry(type_name)?;
            if entry.name_to_slot.contains_key(new_name) {
                return Err(Error::Schema {
                    message: format!(
                        "VOS-DDL-NAME-CONFLICT: `{type_name}.{new_name}` already exists"
                    ),
                    span: None,
                    hint: None,
                });
            }
            let _ = entry.resolve(old_name)?;
        }

        let entry = self.type_entry_mut(type_name)?;
        let slot = *entry.name_to_slot.get(old_name).expect("checked");
        let desc = entry.slots[slot as usize]
            .live_mut()
            .ok_or(Error::Corrupt("rename target is tombstone"))?;
        let field_id = desc.field_id;
        desc.current_name = new_name.to_owned();
        entry.rebuild_name_index();

        // Rewrite display source of macros that bind this FieldId; IR stays put.
        let type_id = self.type_entry(type_name)?.type_id;
        for macro_entry in self.macros.values_mut() {
            let depends = macro_entry
                .field_deps
                .iter()
                .any(|f| f.type_id == type_id && f.field_id == field_id);
            if depends {
                macro_entry.source = rewrite_field_name(&macro_entry.source, old_name, new_name);
                macro_entry.source_hash = blake3_hex(macro_entry.source.as_bytes());
            }
        }

        self.revisions.ddl = self.revisions.ddl.saturating_add(1);
        // Rename does not change observable value types / constraints.
        Ok((field_id, slot))
    }

    /// Add a field — allocates a **new** virtual slot (never a tombstone index).
    pub fn add_field(
        &mut self,
        type_name: &str,
        name: &str,
        ty: TypeExpr,
        attrs: Vec<FieldAttribute>,
    ) -> Result<(FieldId, VirtualFieldIndex)> {
        if name.is_empty() {
            return Err(Error::Schema {
                message: "VOS-DDL-NAME-CONFLICT: field name must be non-empty".into(),
                span: None,
                hint: None,
            });
        }
        {
            let entry = self.type_entry(type_name)?;
            if entry.name_to_slot.contains_key(name) {
                return Err(Error::Schema {
                    message: format!("VOS-DDL-NAME-CONFLICT: `{type_name}.{name}` already exists"),
                    span: None,
                    hint: None,
                });
            }
        }

        let field_id = FieldId(self.alloc_field_id());
        let entry = self.type_entry_mut(type_name)?;
        let source_order = entry
            .slots
            .iter()
            .filter_map(Slot::live)
            .map(|d| d.source_order)
            .max()
            .map(|o| o + 1)
            .unwrap_or(0);
        let slot = entry.slots.len() as VirtualFieldIndex;
        entry.slots.push(Slot::Live(FieldDesc {
            field_id,
            current_name: name.to_owned(),
            source_order,
            ty,
            attrs,
            location: PhysicalLocation::Unbound,
        }));
        entry.rebuild_name_index();

        self.revisions.ddl = self.revisions.ddl.saturating_add(1);
        self.revisions.semantic = self.revisions.semantic.saturating_add(1);
        Ok((field_id, slot))
    }

    /// Drop a field — leaves a tombstone; slot index is never reused.
    ///
    /// `dependents` lists human-readable references; if non-empty, the drop fails.
    pub fn drop_field(
        &mut self,
        type_name: &str,
        field_name: &str,
        dependents: &[&str],
    ) -> Result<VirtualFieldIndex> {
        if !dependents.is_empty() {
            let listed = dependents.join("\n- ");
            return Err(Error::Schema {
                message: format!(
                    "VOS-DDL-FIELD-IN-USE: cannot drop `{type_name}.{field_name}`\n\nreferenced by:\n- {listed}"
                ),
                span: None,
                hint: Some(
                    "update or remove dependents in the same DDL transaction".into(),
                ),
            });
        }

        let entry = self.type_entry_mut(type_name)?;
        let slot = *entry
            .name_to_slot
            .get(field_name)
            .ok_or_else(|| Error::Schema {
                message: format!(
                    "VOS-DDL-UNKNOWN-FIELD: `{type_name}.{field_name}` is not a live field"
                ),
                span: None,
                hint: None,
            })?;
        let desc = entry.slots[slot as usize]
            .live()
            .ok_or(Error::Corrupt("drop target is tombstone"))?
            .clone();
        entry.slots[slot as usize] = Slot::Tombstone {
            field_id: desc.field_id,
            former_name: desc.current_name,
        };
        entry.rebuild_name_index();

        self.revisions.ddl = self.revisions.ddl.saturating_add(1);
        self.revisions.semantic = self.revisions.semantic.saturating_add(1);
        Ok(slot)
    }

    /// Reorder live fields by setting `source_order` from `ordered_names`.
    ///
    /// Virtual indexes and physical locations are unchanged.
    pub fn reorder_fields(&mut self, type_name: &str, ordered_names: &[&str]) -> Result<()> {
        let entry = self.type_entry_mut(type_name)?;
        let live_count = entry.slots.iter().filter(|s| s.live().is_some()).count();
        if ordered_names.len() != live_count {
            return Err(Error::Schema {
                message: format!(
                    "reorder_fields: expected {live_count} live names, got {}",
                    ordered_names.len()
                ),
                span: None,
                hint: None,
            });
        }
        let mut seen = BTreeMap::new();
        for (order, name) in ordered_names.iter().enumerate() {
            let slot = *entry.name_to_slot.get(*name).ok_or_else(|| Error::Schema {
                message: format!("VOS-DDL-UNKNOWN-FIELD: `{type_name}.{name}` is not a live field"),
                span: None,
                hint: None,
            })?;
            if seen.insert(slot, ()).is_some() {
                return Err(Error::Schema {
                    message: format!("duplicate name `{name}` in reorder list"),
                    span: None,
                    hint: None,
                });
            }
            entry.slots[slot as usize]
                .live_mut()
                .expect("live")
                .source_order = order as u32;
        }

        self.revisions.ddl = self.revisions.ddl.saturating_add(1);
        Ok(())
    }

    /// Install or replace a durable macro (DDL session only at the Connection layer).
    pub fn install_macro(
        &mut self,
        name: &str,
        params: &[(String, TypeExpr)],
        return_ty: TypeExpr,
        body_source: &str,
    ) -> Result<MacroId> {
        if name.is_empty() {
            return Err(Error::Schema {
                message: "VOS-DDL-NAME-CONFLICT: macro name must be non-empty".into(),
                span: None,
                hint: None,
            });
        }
        let ir = lower_macro_source(self, params, body_source)?;
        let mut type_deps = BTreeSet::new();
        let mut field_deps = BTreeSet::new();
        for (_, ty) in params {
            if let TypeExpr::Named(tn) = strip_type_optional(ty) {
                if let Ok(entry) = self.type_entry(tn) {
                    type_deps.insert(entry.type_id);
                }
            }
        }
        for load in &ir.field_loads {
            type_deps.insert(load.type_id);
            field_deps.insert(*load);
        }
        let source_hash = blake3_hex(body_source.as_bytes());
        let semantic = semantic_hash_of(&ir);
        let macro_id = if let Some(existing) = self.macros.get(name) {
            existing.macro_id
        } else {
            MacroId(self.alloc_macro_id())
        };
        let revision = self
            .macros
            .get(name)
            .map(|m| m.revision.saturating_add(1))
            .unwrap_or(1);
        self.macros.insert(
            name.to_owned(),
            MacroEntry {
                macro_id,
                name: name.to_owned(),
                params: params.iter().map(|(n, _)| n.clone()).collect(),
                param_types: params.iter().map(|(_, t)| t.clone()).collect(),
                return_ty,
                source: body_source.to_owned(),
                ir,
                type_deps,
                field_deps,
                source_hash,
                semantic_hash: semantic,
                revision,
            },
        );
        self.revisions.ddl = self.revisions.ddl.saturating_add(1);
        self.revisions.semantic = self.revisions.semantic.saturating_add(1);
        Ok(macro_id)
    }

    /// Drop a durable macro by name.
    pub fn drop_macro(&mut self, name: &str) -> Result<MacroId> {
        let entry = self.macros.remove(name).ok_or_else(|| Error::Schema {
            message: format!("VOS-DDL-UNKNOWN-FIELD: macro `{name}` is not defined"),
            span: None,
            hint: None,
        })?;
        self.revisions.ddl = self.revisions.ddl.saturating_add(1);
        self.revisions.semantic = self.revisions.semantic.saturating_add(1);
        Ok(entry.macro_id)
    }

    /// Look up a macro by name.
    pub fn macro_entry(&self, name: &str) -> Result<&MacroEntry> {
        self.macros.get(name).ok_or_else(|| Error::Schema {
            message: format!("VOS-DDL-UNKNOWN-FIELD: macro `{name}` is not defined"),
            span: None,
            hint: None,
        })
    }

    /// Iterate macros.
    pub fn macros(&self) -> impl Iterator<Item = &MacroEntry> {
        self.macros.values()
    }

    /// Human-readable dependents of a live field (macros first).
    pub fn field_dependents(&self, type_name: &str, field_name: &str) -> Result<Vec<String>> {
        let entry = self.type_entry(type_name)?;
        let (field_id, _) = entry.resolve(field_name)?;
        let type_id = entry.type_id;
        let mut out = Vec::new();
        for m in self.macros.values() {
            if m.field_deps
                .iter()
                .any(|f| f.type_id == type_id && f.field_id == field_id)
            {
                out.push(format!("macro {}", m.name));
            }
        }
        Ok(out)
    }

    fn add_type(&mut self, kind: TypeKind, name: &str, fields: &[Field]) -> Result<()> {
        if self.types.contains_key(name) {
            return Err(Error::Schema {
                message: format!("duplicate type `{name}` in catalog"),
                span: None,
                hint: None,
            });
        }
        let type_id = TypeId(self.alloc_type_id());
        let mut slots = Vec::with_capacity(fields.len());
        let mut name_to_slot = BTreeMap::new();
        for (order, field) in fields.iter().enumerate() {
            if name_to_slot.contains_key(&field.name) {
                return Err(Error::Schema {
                    message: format!("duplicate field `{}` on `{name}`", field.name),
                    span: Some((field.span.start, field.span.end)),
                    hint: None,
                });
            }
            let field_id = FieldId(self.alloc_field_id());
            let slot = order as VirtualFieldIndex;
            name_to_slot.insert(field.name.clone(), slot);
            slots.push(Slot::Live(FieldDesc {
                field_id,
                current_name: field.name.clone(),
                source_order: order as u32,
                ty: field.ty.clone(),
                attrs: field.attrs.clone(),
                location: PhysicalLocation::Unbound,
            }));
        }
        self.types.insert(
            name.to_owned(),
            TypeEntry {
                type_id,
                name: name.to_owned(),
                kind,
                slots,
                name_to_slot,
            },
        );
        Ok(())
    }

    /// Regenerate active canonical VOS DDL (live fields only; tombstones omitted).
    ///
    /// Source order follows each field’s `source_order`. Display uses current names.
    pub fn to_canonical_source(&self) -> String {
        let mut out = String::new();
        for entry in self.types.values() {
            let kind = match entry.kind {
                TypeKind::Table => "table",
                TypeKind::Class => "class",
            };
            out.push_str(kind);
            out.push(' ');
            out.push_str(&entry.name);
            out.push_str(" {\n");
            for field in entry.live_fields_in_source_order() {
                out.push_str("    ");
                out.push_str(&format_field_decl(field));
                out.push_str(",\n");
            }
            out.push_str("}\n");
        }
        // Macros are durable in the catalog blob only until the VOS `macro`
        // declaration parses in `vos-parser` (then they join this document).
        out
    }

    /// Encode durable catalog bytes (FieldId / slots / revisions / macros).
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut bytes = CATALOG_MAGIC.to_vec();
        write_u64(&mut bytes, self.revisions.ddl);
        write_u64(&mut bytes, self.revisions.semantic);
        write_u64(&mut bytes, self.revisions.layout_epoch);
        write_u64(&mut bytes, self.next_type_id);
        write_u64(&mut bytes, self.next_field_id);
        write_u64(&mut bytes, self.next_macro_id);
        write_u32(&mut bytes, self.types.len())?;
        for entry in self.types.values() {
            write_u64(&mut bytes, entry.type_id.0);
            bytes.push(match entry.kind {
                TypeKind::Table => 0,
                TypeKind::Class => 1,
            });
            write_str(&mut bytes, &entry.name)?;
            write_u32(&mut bytes, entry.slots.len())?;
            for slot in &entry.slots {
                match slot {
                    Slot::Live(desc) => {
                        bytes.push(0);
                        write_u64(&mut bytes, desc.field_id.0);
                        write_str(&mut bytes, &desc.current_name)?;
                        write_u32(&mut bytes, desc.source_order as usize)?;
                        encode_type_expr(&mut bytes, &desc.ty)?;
                        write_u32(&mut bytes, desc.attrs.len())?;
                        for attr in &desc.attrs {
                            bytes.push(match attr {
                                FieldAttribute::Primary => 1,
                                FieldAttribute::Unique => 2,
                                _ => {
                                    return Err(Error::Unsupported(
                                        "schema catalog encode for unknown FieldAttribute",
                                    ))
                                }
                            });
                        }
                    }
                    Slot::Tombstone {
                        field_id,
                        former_name,
                    } => {
                        bytes.push(1);
                        write_u64(&mut bytes, field_id.0);
                        write_str(&mut bytes, former_name)?;
                    }
                }
            }
        }
        write_u32(&mut bytes, self.macros.len())?;
        for entry in self.macros.values() {
            encode_macro_entry(&mut bytes, entry)?;
        }
        Ok(bytes)
    }

    /// Decode durable catalog bytes written by [`Self::encode`].
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut cursor = 0;
        let magic = take(bytes, &mut cursor, 5)?;
        let version = match magic {
            b"YCAT\x01" => 1u8,
            b"YCAT\x02" => 2u8,
            _ => return Err(Error::Corrupt("unknown schema catalog header")),
        };
        let revisions = Revisions {
            ddl: read_u64(bytes, &mut cursor)?,
            semantic: read_u64(bytes, &mut cursor)?,
            layout_epoch: read_u64(bytes, &mut cursor)?,
        };
        let next_type_id = read_u64(bytes, &mut cursor)?;
        let next_field_id = read_u64(bytes, &mut cursor)?;
        let next_macro_id = if version >= 2 {
            read_u64(bytes, &mut cursor)?
        } else {
            1
        };
        let type_count = read_u32(bytes, &mut cursor)? as usize;
        let mut types = BTreeMap::new();
        for _ in 0..type_count {
            let type_id = TypeId(read_u64(bytes, &mut cursor)?);
            let kind = match take(bytes, &mut cursor, 1)? {
                [0] => TypeKind::Table,
                [1] => TypeKind::Class,
                _ => return Err(Error::Corrupt("unknown type kind in catalog")),
            };
            let name = read_str(bytes, &mut cursor)?;
            let slot_count = read_u32(bytes, &mut cursor)? as usize;
            let mut slots = Vec::with_capacity(slot_count);
            for _ in 0..slot_count {
                match take(bytes, &mut cursor, 1)? {
                    [0] => {
                        let field_id = FieldId(read_u64(bytes, &mut cursor)?);
                        let current_name = read_str(bytes, &mut cursor)?;
                        let source_order = read_u32(bytes, &mut cursor)?;
                        let ty = decode_type_expr(bytes, &mut cursor)?;
                        let attr_count = read_u32(bytes, &mut cursor)? as usize;
                        let mut attrs = Vec::with_capacity(attr_count);
                        for _ in 0..attr_count {
                            match take(bytes, &mut cursor, 1)? {
                                [1] => attrs.push(FieldAttribute::Primary),
                                [2] => attrs.push(FieldAttribute::Unique),
                                _ => {
                                    return Err(Error::Corrupt(
                                        "unknown field attribute in catalog",
                                    ))
                                }
                            }
                        }
                        slots.push(Slot::Live(FieldDesc {
                            field_id,
                            current_name,
                            source_order,
                            ty,
                            attrs,
                            location: PhysicalLocation::Unbound,
                        }));
                    }
                    [1] => {
                        let field_id = FieldId(read_u64(bytes, &mut cursor)?);
                        let former_name = read_str(bytes, &mut cursor)?;
                        slots.push(Slot::Tombstone {
                            field_id,
                            former_name,
                        });
                    }
                    _ => return Err(Error::Corrupt("unknown slot tag in catalog")),
                }
            }
            let mut entry = TypeEntry {
                type_id,
                name: name.clone(),
                kind,
                slots,
                name_to_slot: BTreeMap::new(),
            };
            entry.rebuild_name_index();
            types.insert(name, entry);
        }
        let mut macros = BTreeMap::new();
        if version >= 2 {
            let macro_count = read_u32(bytes, &mut cursor)? as usize;
            for _ in 0..macro_count {
                let entry = decode_macro_entry(bytes, &mut cursor)?;
                macros.insert(entry.name.clone(), entry);
            }
        }
        if cursor != bytes.len() {
            return Err(Error::Corrupt("trailing data in schema catalog"));
        }
        Ok(Self {
            revisions,
            types,
            macros,
            next_type_id,
            next_field_id,
            next_macro_id,
        })
    }

    fn alloc_type_id(&mut self) -> u64 {
        let id = self.next_type_id;
        self.next_type_id = self.next_type_id.saturating_add(1);
        id
    }

    fn alloc_field_id(&mut self) -> u64 {
        let id = self.next_field_id;
        self.next_field_id = self.next_field_id.saturating_add(1);
        id
    }

    fn alloc_macro_id(&mut self) -> u64 {
        let id = self.next_macro_id;
        self.next_macro_id = self.next_macro_id.saturating_add(1);
        id
    }
}

const CATALOG_MAGIC: &[u8] = b"YCAT\x02";

fn strip_type_optional(ty: &TypeExpr) -> &TypeExpr {
    match ty {
        TypeExpr::Optional(inner) => strip_type_optional(inner),
        other => other,
    }
}

fn encode_macro_entry(bytes: &mut Vec<u8>, entry: &MacroEntry) -> Result<()> {
    write_u64(bytes, entry.macro_id.0);
    write_str(bytes, &entry.name)?;
    write_u32(bytes, entry.params.len())?;
    for (name, ty) in entry.params.iter().zip(entry.param_types.iter()) {
        write_str(bytes, name)?;
        encode_type_expr(bytes, ty)?;
    }
    encode_type_expr(bytes, &entry.return_ty)?;
    write_str(bytes, &entry.source)?;
    write_u32(bytes, entry.ir.field_loads.len())?;
    for load in &entry.ir.field_loads {
        write_u64(bytes, load.type_id.0);
        write_u64(bytes, load.field_id.0);
        write_u32(bytes, load.virtual_field as usize)?;
    }
    write_str(bytes, &entry.source_hash)?;
    write_str(bytes, &entry.semantic_hash)?;
    write_u64(bytes, entry.revision);
    Ok(())
}

fn decode_macro_entry(bytes: &[u8], cursor: &mut usize) -> Result<MacroEntry> {
    let macro_id = MacroId(read_u64(bytes, cursor)?);
    let name = read_str(bytes, cursor)?;
    let param_count = read_u32(bytes, cursor)? as usize;
    let mut params = Vec::with_capacity(param_count);
    let mut param_types = Vec::with_capacity(param_count);
    for _ in 0..param_count {
        params.push(read_str(bytes, cursor)?);
        param_types.push(decode_type_expr(bytes, cursor)?);
    }
    let return_ty = decode_type_expr(bytes, cursor)?;
    let source = read_str(bytes, cursor)?;
    let load_count = read_u32(bytes, cursor)? as usize;
    let mut field_loads = Vec::with_capacity(load_count);
    let mut type_deps = BTreeSet::new();
    let mut field_deps = BTreeSet::new();
    for _ in 0..load_count {
        let load = FieldRef {
            type_id: TypeId(read_u64(bytes, cursor)?),
            field_id: FieldId(read_u64(bytes, cursor)?),
            virtual_field: read_u32(bytes, cursor)?,
        };
        type_deps.insert(load.type_id);
        field_deps.insert(load);
        field_loads.push(load);
    }
    let source_hash = read_str(bytes, cursor)?;
    let semantic_hash = read_str(bytes, cursor)?;
    let revision = read_u64(bytes, cursor)?;
    Ok(MacroEntry {
        macro_id,
        name,
        params,
        param_types,
        return_ty,
        source,
        ir: MacroIr { field_loads },
        type_deps,
        field_deps,
        source_hash,
        semantic_hash,
        revision,
    })
}

fn format_field_decl(field: &FieldDesc) -> String {
    let ty = format_type_expr(&field.ty);
    let has_primary = field.attrs.contains(&FieldAttribute::Primary);
    let has_unique = field.attrs.contains(&FieldAttribute::Unique);
    if has_primary && !has_unique {
        format!("@@{}: {ty}", field.current_name)
    } else if has_unique && !has_primary {
        format!("@{}: {ty}", field.current_name)
    } else if has_primary {
        format!("[primary] {}: {ty}", field.current_name)
    } else {
        format!("{}: {ty}", field.current_name)
    }
}

fn format_type_expr(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Builtin(b) => match b {
            BuiltinType::DateTimeUtc => "datetime".to_owned(),
            other => other.as_vos().to_owned(),
        },
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::Reference(inner) => format!("&{}", format_type_expr(inner)),
        TypeExpr::Optional(inner) => format!("{}?", format_type_expr(inner)),
        TypeExpr::List(inner) => format!("[{}]", format_type_expr(inner)),
        TypeExpr::Vector { dim } => format!("vector<{dim}>"),
        TypeExpr::File => "file".to_owned(),
        _ => "utf8".to_owned(),
    }
}

fn builtin_wire_name(b: BuiltinType) -> &'static str {
    match b {
        BuiltinType::DateTimeUtc => "datetime",
        other => other.as_vos(),
    }
}

fn encode_type_expr(bytes: &mut Vec<u8>, ty: &TypeExpr) -> Result<()> {
    match ty {
        TypeExpr::Builtin(b) => {
            bytes.push(1);
            write_str(bytes, builtin_wire_name(*b))?;
        }
        TypeExpr::Named(name) => {
            bytes.push(2);
            write_str(bytes, name)?;
        }
        TypeExpr::Reference(inner) => {
            bytes.push(3);
            encode_type_expr(bytes, inner)?;
        }
        TypeExpr::Optional(inner) => {
            bytes.push(4);
            encode_type_expr(bytes, inner)?;
        }
        TypeExpr::List(inner) => {
            bytes.push(5);
            encode_type_expr(bytes, inner)?;
        }
        TypeExpr::Vector { dim } => {
            bytes.push(6);
            write_u32(bytes, *dim as usize)?;
        }
        TypeExpr::File => bytes.push(7),
        _ => {
            return Err(Error::Unsupported(
                "schema catalog encode for unknown TypeExpr variant",
            ))
        }
    }
    Ok(())
}

fn decode_type_expr(bytes: &[u8], cursor: &mut usize) -> Result<TypeExpr> {
    match take(bytes, cursor, 1)? {
        [1] => {
            let name = read_str(bytes, cursor)?;
            let builtin = BuiltinType::parse(&name).ok_or(Error::Corrupt(
                "unknown builtin in schema catalog type expr",
            ))?;
            Ok(TypeExpr::Builtin(builtin))
        }
        [2] => Ok(TypeExpr::Named(read_str(bytes, cursor)?)),
        [3] => Ok(TypeExpr::Reference(Box::new(decode_type_expr(
            bytes, cursor,
        )?))),
        [4] => Ok(TypeExpr::Optional(Box::new(decode_type_expr(
            bytes, cursor,
        )?))),
        [5] => Ok(TypeExpr::List(Box::new(decode_type_expr(bytes, cursor)?))),
        [6] => Ok(TypeExpr::Vector {
            dim: read_u32(bytes, cursor)?,
        }),
        [7] => Ok(TypeExpr::File),
        _ => Err(Error::Corrupt("unknown type expr tag in catalog")),
    }
}

fn write_u32(bytes: &mut Vec<u8>, value: usize) -> Result<()> {
    let value = u32::try_from(value).map_err(|_| Error::Corrupt("value exceeds 4 GiB"))?;
    bytes.extend(value.to_le_bytes());
    Ok(())
}

fn write_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend(value.to_le_bytes());
}

fn write_str(bytes: &mut Vec<u8>, value: &str) -> Result<()> {
    write_u32(bytes, value.len())?;
    bytes.extend(value.as_bytes());
    Ok(())
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32> {
    let raw = take(bytes, cursor, 4)?;
    Ok(u32::from_le_bytes(
        raw.try_into().expect("requested exactly 4 bytes"),
    ))
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64> {
    let raw = take(bytes, cursor, 8)?;
    Ok(u64::from_le_bytes(
        raw.try_into().expect("requested exactly 8 bytes"),
    ))
}

fn read_str(bytes: &[u8], cursor: &mut usize) -> Result<String> {
    let len = read_u32(bytes, cursor)? as usize;
    let raw = take(bytes, cursor, len)?;
    String::from_utf8(raw.to_vec()).map_err(|_| Error::Corrupt("catalog string is not UTF-8"))
}

fn take<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> Result<&'a [u8]> {
    let end = cursor
        .checked_add(len)
        .ok_or(Error::Corrupt("catalog overflow"))?;
    if end > bytes.len() {
        return Err(Error::Corrupt("truncated catalog"));
    }
    let slice = &bytes[*cursor..end];
    *cursor = end;
    Ok(slice)
}

impl Default for SchemaCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vos::ast::BuiltinType;
    use vos::parser::parse_document;

    fn user_doc() -> vos::ast::Document {
        parse_document(
            r#"
            table User {
                @@user_id: uuid,
                @user_name: utf8,
                avatar: utf8?,
            }
            "#,
        )
        .unwrap()
    }

    #[test]
    fn from_document_assigns_stable_slots() {
        let catalog = SchemaCatalog::from_document(&user_doc()).unwrap();
        let user = catalog.type_entry("User").unwrap();
        assert_eq!(user.slots.len(), 3);
        let (id0, slot0) = user.resolve("user_id").unwrap();
        let (id1, slot1) = user.resolve("user_name").unwrap();
        assert_eq!(slot0, 0);
        assert_eq!(slot1, 1);
        assert_ne!(id0, id1);
        assert_eq!(catalog.revisions.ddl, 1);
    }

    #[test]
    fn encode_roundtrip_preserves_ids_and_tombstones() {
        let mut catalog = SchemaCatalog::from_document(&user_doc()).unwrap();
        catalog
            .rename_field("User", "user_name", "display_name")
            .unwrap();
        catalog.drop_field("User", "avatar", &[]).unwrap();
        let bytes = catalog.encode().unwrap();
        let restored = SchemaCatalog::decode(&bytes).unwrap();
        assert_eq!(restored.revisions, catalog.revisions);
        let (id, slot) = restored
            .type_entry("User")
            .unwrap()
            .resolve("display_name")
            .unwrap();
        assert_eq!(slot, 1);
        assert_eq!(
            id,
            catalog
                .type_entry("User")
                .unwrap()
                .resolve("display_name")
                .unwrap()
                .0
        );
        assert!(matches!(
            restored.type_entry("User").unwrap().slots[2],
            Slot::Tombstone { .. }
        ));
        let src = restored.to_canonical_source();
        assert!(src.contains("display_name"));
        assert!(!src.contains("avatar"));
        parse_document(&src).unwrap();
    }

    #[test]
    fn rename_preserves_field_id_and_slot() {
        let mut catalog = SchemaCatalog::from_document(&user_doc()).unwrap();
        let (before_id, before_slot) = catalog
            .type_entry("User")
            .unwrap()
            .resolve("user_name")
            .unwrap();
        let (after_id, after_slot) = catalog
            .rename_field("User", "user_name", "display_name")
            .unwrap();
        assert_eq!(before_id, after_id);
        assert_eq!(before_slot, after_slot);
        assert!(catalog
            .type_entry("User")
            .unwrap()
            .resolve("user_name")
            .is_err());
        assert_eq!(
            catalog.type_entry("User").unwrap().name_at(after_slot),
            Some("display_name")
        );
        assert_eq!(catalog.revisions.ddl, 2);
        assert_eq!(catalog.revisions.semantic, 1);
    }

    #[test]
    fn drop_leaves_tombstone_and_add_does_not_reuse_slot() {
        let mut catalog = SchemaCatalog::from_document(&user_doc()).unwrap();
        let dropped = catalog.drop_field("User", "user_name", &[]).unwrap();
        assert_eq!(dropped, 1);
        let user = catalog.type_entry("User").unwrap();
        assert!(matches!(user.slots[1], Slot::Tombstone { .. }));
        assert!(user.resolve("user_name").is_err());

        let (_id, email_slot) = catalog
            .add_field(
                "User",
                "email",
                TypeExpr::Optional(Box::new(TypeExpr::Builtin(BuiltinType::Utf8))),
                vec![],
            )
            .unwrap();
        assert_eq!(email_slot, 3); // slots 0,1(tomb),2, then 3
        assert_eq!(catalog.type_entry("User").unwrap().slots.len(), 4);
    }

    #[test]
    fn drop_with_dependents_fails() {
        let mut catalog = SchemaCatalog::from_document(&user_doc()).unwrap();
        let err = catalog
            .drop_field("User", "user_name", &["macro public_user"])
            .unwrap_err();
        match err {
            Error::Schema { message, .. } => {
                assert!(message.contains("VOS-DDL-FIELD-IN-USE"));
                assert!(message.contains("public_user"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn reorder_only_changes_source_order() {
        let mut catalog = SchemaCatalog::from_document(&user_doc()).unwrap();
        let (id, slot) = catalog
            .type_entry("User")
            .unwrap()
            .resolve("avatar")
            .unwrap();
        catalog
            .reorder_fields("User", &["avatar", "user_id", "user_name"])
            .unwrap();
        let user = catalog.type_entry("User").unwrap();
        assert_eq!(user.resolve("avatar").unwrap(), (id, slot));
        let ordered: Vec<_> = user
            .live_fields_in_source_order()
            .iter()
            .map(|d| d.current_name.as_str())
            .collect();
        assert_eq!(ordered, vec!["avatar", "user_id", "user_name"]);
    }
}
