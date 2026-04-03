use futures_util::TryStreamExt;
use mongodb::{
    bson::{self, Bson, Document, doc, oid::ObjectId},
    error::Error as MongoError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::catalog::dto::{
    CreateDocumentLocaleRequest, CreateDocumentRequest, CreateLocalizedDocumentRequest,
    CreateLocalizedDocumentResponse, CreateSchemaRequest, DocumentView, EntryView,
    SchemaFieldDefinition, SchemaFieldType, SchemaI18nConfig, SchemaI18nMode, SchemaView,
    UpdateDocumentRequest, UpdateSchemaFieldsRequest, UpdateSchemaRequest,
};

const SCHEMA_COLLECTION: &str = "_schemas";

#[derive(Clone, Debug)]
pub struct ActorStamp {
    pub user_id: i64,
    pub username: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SchemaRecord {
    #[serde(rename = "_id")]
    id: ObjectId,
    key: String,
    display_name: String,
    description: Option<String>,
    i18n: Option<SchemaI18nConfig>,
    fields: Vec<SchemaFieldDefinition>,
    created_at: i64,
    updated_at: i64,
}

pub async fn list_schemas() -> Result<Vec<SchemaView>, String> {
    let collection = crate::mongo::database().collection::<Document>(SCHEMA_COLLECTION);
    let cursor = collection
        .find(doc! {})
        .sort(doc! { "updated_at": -1_i32 })
        .await
        .map_err(map_mongo_error)?;
    let docs: Vec<Document> = cursor.try_collect().await.map_err(map_mongo_error)?;
    docs.into_iter().map(map_schema_document).collect()
}

pub async fn create_schema(payload: CreateSchemaRequest) -> Result<SchemaView, String> {
    let key = normalize_schema_key(&payload.key)?;
    let display_name = normalize_required(&payload.display_name, "schema_display_name_required")?;
    let fields = normalize_fields(payload.fields.unwrap_or_default())?;
    let now = crate::admin::middlewares::auth::unix_timestamp();

    let collection = crate::mongo::database().collection::<SchemaRecord>(SCHEMA_COLLECTION);
    let record = SchemaRecord {
        id: ObjectId::new(),
        key,
        display_name,
        description: normalize_optional(payload.description),
        i18n: normalize_i18n_config(payload.i18n, &fields)?,
        fields,
        created_at: now,
        updated_at: now,
    };

    collection.insert_one(&record).await.map_err(|error| {
        if error.to_string().contains("E11000") {
            "schema_key_taken".to_string()
        } else {
            map_mongo_error(error)
        }
    })?;
    Ok(map_schema_view(record))
}

pub async fn get_schema(id: &str) -> Result<SchemaView, String> {
    let object_id = parse_object_id(id)?;
    let collection = crate::mongo::database().collection::<Document>(SCHEMA_COLLECTION);
    let record = collection
        .find_one(doc! { "_id": object_id })
        .await
        .map_err(map_mongo_error)?
        .ok_or_else(|| "schema_not_found".to_string())?;
    map_schema_document(record)
}

pub async fn update_schema(id: &str, payload: UpdateSchemaRequest) -> Result<SchemaView, String> {
    let object_id = parse_object_id(id)?;
    let collection = crate::mongo::database().collection::<SchemaRecord>(SCHEMA_COLLECTION);
    let current = collection
        .find_one(doc! { "_id": object_id })
        .await
        .map_err(map_mongo_error)?
        .ok_or_else(|| "schema_not_found".to_string())?;

    let next_display_name = payload
        .display_name
        .as_deref()
        .map(|value| normalize_required(value, "schema_display_name_required"))
        .transpose()?
        .unwrap_or(current.display_name);
    let next_description = payload
        .description
        .map(|value| normalize_optional(Some(value)))
        .unwrap_or(current.description);
    let next_i18n = match payload.i18n {
        Some(i18n) => normalize_i18n_config(Some(i18n), &current.fields)?,
        None => current.i18n,
    };
    let now = crate::admin::middlewares::auth::unix_timestamp();

    collection
        .update_one(
            doc! { "_id": object_id },
            doc! {
                "$set": {
                    "display_name": next_display_name,
                    "description": bson::to_bson(&next_description).map_err(|error| error.to_string())?,
                    "i18n": bson::to_bson(&next_i18n).map_err(|error| error.to_string())?,
                    "updated_at": now,
                }
            },
        )
        .await
        .map_err(map_mongo_error)?;
    get_schema(id).await
}

pub async fn update_schema_fields(
    id: &str,
    payload: UpdateSchemaFieldsRequest,
) -> Result<SchemaView, String> {
    let object_id = parse_object_id(id)?;
    let fields = normalize_fields(payload.fields)?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let collection = crate::mongo::database().collection::<SchemaRecord>(SCHEMA_COLLECTION);
    let fields_bson = bson::to_bson(&fields).map_err(|error| error.to_string())?;
    let current = collection
        .find_one(doc! { "_id": object_id })
        .await
        .map_err(map_mongo_error)?
        .ok_or_else(|| "schema_not_found".to_string())?;
    let i18n = normalize_i18n_config(current.i18n, &fields)?;
    collection
        .update_one(
            doc! { "_id": object_id },
            doc! {
                "$set": {
                    "fields": fields_bson,
                    "i18n": bson::to_bson(&i18n).map_err(|error| error.to_string())?,
                    "updated_at": now,
                }
            },
        )
        .await
        .map_err(map_mongo_error)?;
    get_schema(id).await
}

pub async fn list_documents(
    schema_key: &str,
    parent_id: Option<String>,
    keyword: Option<String>,
    enabled: Option<bool>,
    status: Option<String>,
    page: Option<u64>,
    page_size: Option<u64>,
) -> Result<Vec<DocumentView>, String> {
    let schema_key = normalize_schema_key(schema_key)?;
    let collection = crate::mongo::database().collection::<Document>(&schema_key);
    let mut filter = doc! { "schema_key": &schema_key };

    if let Some(parent_id) = parent_id {
        filter.insert("parent_id", parent_id);
    }
    if let Some(enabled) = enabled {
        filter.insert("enabled", enabled);
    }
    if let Some(status) = status.filter(|value| !value.trim().is_empty()) {
        filter.insert("status", status);
    }
    if let Some(keyword) = keyword.filter(|value| !value.trim().is_empty()) {
        filter.insert("search_text", doc! { "$regex": keyword, "$options": "i" });
    }

    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(20).clamp(1, 100);

    let cursor = collection
        .find(filter)
        .sort(doc! { "updated_at": -1_i32 })
        .skip((page - 1) * page_size)
        .limit(page_size as i64)
        .await
        .map_err(map_mongo_error)?;
    let docs: Vec<Document> = cursor.try_collect().await.map_err(map_mongo_error)?;
    docs.into_iter()
        .map(|doc| map_document_view(&schema_key, doc))
        .collect()
}

pub async fn create_document(
    schema_key: &str,
    payload: CreateDocumentRequest,
    actor: Option<ActorStamp>,
) -> Result<DocumentView, String> {
    let schema = get_schema_by_key(schema_key).await?;
    create_document_with_schema(&schema, payload, actor).await
}

pub async fn create_localized_document(
    schema_key: &str,
    payload: CreateLocalizedDocumentRequest,
    actor: Option<ActorStamp>,
) -> Result<CreateLocalizedDocumentResponse, String> {
    let localized_schema = get_schema_by_key(schema_key).await?;
    let Some(i18n) = &localized_schema.i18n else {
        return Err("schema_i18n_not_configured".to_string());
    };
    if i18n.mode != SchemaI18nMode::Translation {
        return Err("schema_is_not_translation".to_string());
    }

    let root_schema_key = i18n
        .root_schema_key
        .clone()
        .ok_or_else(|| "root_schema_key_required".to_string())?;
    let locale_field = i18n
        .locale_field
        .clone()
        .ok_or_else(|| "locale_field_required".to_string())?;
    let root_schema = get_schema_by_key(&root_schema_key).await?;

    let root_document = create_document_with_schema(
        &root_schema,
        CreateDocumentRequest {
            parent_id: None,
            status: payload.root_status,
            enabled: payload.root_enabled,
            fields: payload.root_fields.unwrap_or_else(empty_object),
        },
        actor.clone(),
    )
    .await?;

    let localized_fields = inject_locale_field(payload.fields, &locale_field, payload.locale)?;
    let localized_result = create_document_with_schema(
        &localized_schema,
        CreateDocumentRequest {
            parent_id: Some(root_document.id.clone()),
            status: payload.localized_status,
            enabled: payload.localized_enabled,
            fields: localized_fields,
        },
        actor,
    )
    .await;

    match localized_result {
        Ok(localized_document) => Ok(CreateLocalizedDocumentResponse {
            root_document,
            localized_document,
        }),
        Err(error) => {
            let root_collection = crate::mongo::database().collection::<Document>(&root_schema.key);
            let root_object_id = ObjectId::parse_str(&root_document.id)
                .map_err(|parse_error| parse_error.to_string())?;
            let _ = root_collection
                .delete_one(doc! { "_id": root_object_id })
                .await;
            Err(error)
        }
    }
}

pub async fn list_entries(
    root_schema_key: &str,
    locale: &str,
    enabled: Option<bool>,
    status: Option<String>,
    keyword: Option<String>,
    page: Option<u64>,
    page_size: Option<u64>,
) -> Result<Vec<EntryView>, String> {
    let root_schema = get_schema_by_key(root_schema_key).await?;
    let translation_schema = find_translation_schema_by_root(&root_schema.key).await?;
    let locale = normalize_locale(locale)?;
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(20).clamp(1, 100);

    let mut entries = Vec::new();
    for root_document in
        list_documents(&root_schema.key, None, None, enabled, status, None, None).await?
    {
        let localized_document =
            find_localized_document(&translation_schema, &root_document.id, &locale).await?;
        let available_locales =
            list_available_locales(&translation_schema, &root_document.id).await?;
        entries.push(EntryView {
            locale: locale.clone(),
            translation_schema_key: translation_schema.key.clone(),
            root_document,
            localized_document,
            available_locales,
        });
    }

    if let Some(keyword) = keyword.filter(|value| !value.trim().is_empty()) {
        let keyword = keyword.to_ascii_lowercase();
        entries.retain(|entry| {
            let root_match = entry
                .root_document
                .fields
                .to_string()
                .to_ascii_lowercase()
                .contains(&keyword);
            let localized_match = entry.localized_document.as_ref().is_some_and(|document| {
                document
                    .fields
                    .to_string()
                    .to_ascii_lowercase()
                    .contains(&keyword)
            });
            root_match || localized_match
        });
    }

    let start = ((page - 1) * page_size) as usize;
    if start >= entries.len() {
        return Ok(Vec::new());
    }
    let end = start.saturating_add(page_size as usize).min(entries.len());
    Ok(entries.into_iter().skip(start).take(end - start).collect())
}

pub async fn get_entry_detail(
    root_schema_key: &str,
    root_document_id: &str,
    locale: &str,
) -> Result<EntryView, String> {
    let root_schema = get_schema_by_key(root_schema_key).await?;
    let translation_schema = find_translation_schema_by_root(&root_schema.key).await?;
    let locale = normalize_locale(locale)?;
    let root_document = get_document(&root_schema.key, root_document_id).await?;
    let localized_document =
        find_localized_document(&translation_schema, &root_document.id, &locale).await?;
    let available_locales = list_available_locales(&translation_schema, &root_document.id).await?;

    Ok(EntryView {
        locale,
        translation_schema_key: translation_schema.key,
        root_document,
        localized_document,
        available_locales,
    })
}

pub async fn create_document_locale(
    translation_schema_key: &str,
    root_document_id: &str,
    payload: CreateDocumentLocaleRequest,
    actor: Option<ActorStamp>,
) -> Result<DocumentView, String> {
    let translation_schema = get_schema_by_key(translation_schema_key).await?;
    let Some(i18n) = &translation_schema.i18n else {
        return Err("schema_i18n_not_configured".to_string());
    };
    if i18n.mode != SchemaI18nMode::Translation {
        return Err("schema_is_not_translation".to_string());
    }
    let root_schema_key = i18n
        .root_schema_key
        .clone()
        .ok_or_else(|| "root_schema_key_required".to_string())?;
    let locale_field = i18n
        .locale_field
        .clone()
        .ok_or_else(|| "locale_field_required".to_string())?;

    let root_document = get_document(&root_schema_key, root_document_id).await?;
    let locale = normalize_locale(&payload.locale)?;
    if find_localized_document(&translation_schema, &root_document.id, &locale)
        .await?
        .is_some()
    {
        return Err("localized_document_already_exists".to_string());
    }

    let fields = inject_locale_field(payload.fields, &locale_field, locale)?;
    create_document_with_schema(
        &translation_schema,
        CreateDocumentRequest {
            parent_id: Some(root_document.id),
            status: payload.localized_status,
            enabled: payload.localized_enabled,
            fields,
        },
        actor,
    )
    .await
}

pub async fn get_document(schema_key: &str, document_id: &str) -> Result<DocumentView, String> {
    let schema = get_schema_by_key(schema_key).await?;
    let collection = crate::mongo::database().collection::<Document>(&schema.key);
    let object_id = parse_object_id(document_id)?;
    let document = collection
        .find_one(doc! { "_id": object_id, "schema_key": &schema.key })
        .await
        .map_err(map_mongo_error)?
        .ok_or_else(|| "document_not_found".to_string())?;
    map_document_view(&schema.key, document)
}

pub async fn update_document(
    schema_key: &str,
    document_id: &str,
    payload: UpdateDocumentRequest,
    actor: Option<ActorStamp>,
) -> Result<DocumentView, String> {
    let schema = get_schema_by_key(schema_key).await?;
    let collection = crate::mongo::database().collection::<Document>(&schema.key);
    let object_id = parse_object_id(document_id)?;
    let current = collection
        .find_one(doc! { "_id": object_id, "schema_key": &schema.key })
        .await
        .map_err(map_mongo_error)?
        .ok_or_else(|| "document_not_found".to_string())?;

    let current_fields = current.get_document("fields").cloned().unwrap_or_default();
    let merged_fields = if let Some(fields) = payload.fields {
        validate_document_fields(
            &schema.fields,
            merge_json_objects(bson_document_to_json_value(current_fields.clone())?, fields)?,
            false,
        )?
    } else {
        bson_document_to_json_value(current_fields)?
    };

    let now = crate::admin::middlewares::auth::unix_timestamp();
    let fields_document = json_object_to_bson_document(&merged_fields)?;
    let search_text = build_search_text(&schema.fields, &merged_fields);

    collection
        .update_one(
            doc! { "_id": object_id, "schema_key": &schema.key },
            doc! {
                "$set": {
                    "parent_id": bson::to_bson(&payload.parent_id).map_err(|error| error.to_string())?,
                    "status": payload.status.unwrap_or_else(|| current.get_str("status").unwrap_or("draft").to_string()),
                    "enabled": payload.enabled.unwrap_or_else(|| current.get_bool("enabled").unwrap_or(true)),
                    "updated_at": now,
                    "updated_by": actor.as_ref().map(|value| value.username.clone()),
                    "updated_by_user_id": actor.as_ref().map(|value| value.user_id),
                    "fields": fields_document,
                    "search_text": search_text,
                }
            },
        )
        .await
        .map_err(map_mongo_error)?;

    get_document(schema_key, document_id).await
}

pub async fn delete_document(schema_key: &str, document_id: &str) -> Result<(), String> {
    let schema = get_schema_by_key(schema_key).await?;
    let collection = crate::mongo::database().collection::<Document>(&schema.key);
    let object_id = parse_object_id(document_id)?;
    let result = collection
        .delete_one(doc! { "_id": object_id, "schema_key": &schema.key })
        .await
        .map_err(map_mongo_error)?;
    if result.deleted_count == 0 {
        return Err("document_not_found".to_string());
    }
    Ok(())
}

async fn get_schema_by_key(schema_key: &str) -> Result<SchemaRecord, String> {
    let collection = crate::mongo::database().collection::<Document>(SCHEMA_COLLECTION);
    let document = collection
        .find_one(doc! { "key": normalize_schema_key(schema_key)? })
        .await
        .map_err(map_mongo_error)?
        .ok_or_else(|| "schema_not_found".to_string())?;
    map_schema_record(document)
}

async fn find_translation_schema_by_root(root_schema_key: &str) -> Result<SchemaRecord, String> {
    let normalized_root_schema_key = normalize_schema_key(root_schema_key)?;
    let collection = crate::mongo::database().collection::<Document>(SCHEMA_COLLECTION);
    let cursor = collection
        .find(doc! {
            "i18n.mode": "translation",
            "i18n.root_schema_key": &normalized_root_schema_key,
        })
        .await
        .map_err(map_mongo_error)?;
    let schemas: Vec<Document> = cursor.try_collect().await.map_err(map_mongo_error)?;

    let mut schemas = schemas
        .into_iter()
        .map(map_schema_record)
        .collect::<Result<Vec<_>, _>>()?;
    match schemas.len() {
        0 => Err("translation_schema_not_found".to_string()),
        1 => Ok(schemas.remove(0)),
        _ => Err("multiple_translation_schemas_found".to_string()),
    }
}

fn map_schema_view(record: SchemaRecord) -> SchemaView {
    SchemaView {
        id: record.id.to_hex(),
        key: record.key,
        display_name: record.display_name,
        description: record.description,
        i18n: record.i18n,
        fields: record.fields,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn map_schema_document(document: Document) -> Result<SchemaView, String> {
    Ok(map_schema_view(map_schema_record(document)?))
}

fn map_schema_record(document: Document) -> Result<SchemaRecord, String> {
    bson::from_document(document).map_err(|error| error.to_string())
}

fn map_document_view(schema_key: &str, document: Document) -> Result<DocumentView, String> {
    let id = document
        .get_object_id("_id")
        .map_err(|error| error.to_string())?
        .to_hex();
    let parent_id = document.get_str("parent_id").ok().map(str::to_string);
    let fields = document.get_document("fields").cloned().unwrap_or_default();
    Ok(DocumentView {
        id,
        schema_key: schema_key.to_string(),
        parent_id,
        status: document.get_str("status").unwrap_or("draft").to_string(),
        enabled: document.get_bool("enabled").unwrap_or(true),
        created_at: document.get_i64("created_at").unwrap_or_default(),
        updated_at: document.get_i64("updated_at").unwrap_or_default(),
        created_by: document.get_str("created_by").ok().map(str::to_string),
        updated_by: document.get_str("updated_by").ok().map(str::to_string),
        fields: bson_document_to_json_value(fields)?,
    })
}

fn normalize_schema_key(value: &str) -> Result<String, String> {
    let normalized = value
        .trim()
        .split('?')
        .next()
        .unwrap_or(value)
        .trim()
        .to_ascii_lowercase()
        .replace(' ', "_");
    if normalized.is_empty() {
        return Err("schema_key_required".to_string());
    }
    if normalized.starts_with('_') {
        return Err("schema_key_reserved".to_string());
    }
    Ok(normalized)
}

fn normalize_required(value: &str, error: &str) -> Result<String, String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(error.to_string());
    }
    Ok(normalized)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let normalized = value.trim().to_string();
        (!normalized.is_empty()).then_some(normalized)
    })
}

fn normalize_locale(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    if normalized.is_empty() {
        return Err("locale_required".to_string());
    }
    Ok(normalized)
}

fn normalize_i18n_config(
    value: Option<SchemaI18nConfig>,
    fields: &[SchemaFieldDefinition],
) -> Result<Option<SchemaI18nConfig>, String> {
    let Some(config) = value else {
        return Ok(None);
    };

    match config.mode {
        SchemaI18nMode::Neutral => Ok(Some(SchemaI18nConfig {
            mode: SchemaI18nMode::Neutral,
            root_schema_key: None,
            locale_field: None,
        })),
        SchemaI18nMode::Root => Ok(Some(SchemaI18nConfig {
            mode: SchemaI18nMode::Root,
            root_schema_key: None,
            locale_field: None,
        })),
        SchemaI18nMode::Translation => {
            let root_schema_key = config
                .root_schema_key
                .as_deref()
                .map(normalize_schema_key)
                .transpose()?
                .ok_or_else(|| "root_schema_key_required".to_string())?;
            let locale_field = config
                .locale_field
                .as_deref()
                .map(|value| normalize_schema_key(value))
                .transpose()?
                .ok_or_else(|| "locale_field_required".to_string())?;

            let locale_definition = fields
                .iter()
                .find(|field| field.key == locale_field)
                .ok_or_else(|| "locale_field_missing".to_string())?;
            if locale_definition.field_type != SchemaFieldType::String {
                return Err("locale_field_must_be_string".to_string());
            }

            Ok(Some(SchemaI18nConfig {
                mode: SchemaI18nMode::Translation,
                root_schema_key: Some(root_schema_key),
                locale_field: Some(locale_field),
            }))
        }
    }
}

fn normalize_fields(
    fields: Vec<SchemaFieldDefinition>,
) -> Result<Vec<SchemaFieldDefinition>, String> {
    let mut keys = std::collections::BTreeSet::new();
    let mut normalized = Vec::with_capacity(fields.len());
    for field in fields {
        let key = normalize_schema_key(&field.key)?;
        if !keys.insert(key.clone()) {
            return Err(format!("schema field key 重复: {key}"));
        }
        normalized.push(SchemaFieldDefinition {
            key,
            label: normalize_required(&field.label, "schema_field_label_required")?,
            field_type: field.field_type,
            required: field.required,
            searchable: field.searchable,
            sortable: field.sortable,
            default_value: field.default_value,
            options: field.options,
            references: normalize_optional(field.references),
            order: field.order,
        });
    }
    normalized.sort_by_key(|field| field.order);
    Ok(normalized)
}

fn validate_document_fields(
    definitions: &[SchemaFieldDefinition],
    fields: Value,
    is_create: bool,
) -> Result<Value, String> {
    let mut map = fields
        .as_object()
        .cloned()
        .ok_or_else(|| "document_fields_must_be_object".to_string())?;

    for key in map.keys() {
        if !definitions.iter().any(|field| field.key == *key) {
            return Err(format!("undeclared_field:{key}"));
        }
    }

    for definition in definitions {
        match map.get(&definition.key) {
            Some(value) => ensure_field_type(definition, value)?,
            None => {
                if let Some(default_value) = &definition.default_value {
                    map.insert(definition.key.clone(), default_value.clone());
                } else if definition.required && is_create {
                    return Err(format!("required_field_missing:{}", definition.key));
                }
            }
        }
    }

    Ok(Value::Object(map))
}

async fn create_document_with_schema(
    schema: &SchemaRecord,
    payload: CreateDocumentRequest,
    actor: Option<ActorStamp>,
) -> Result<DocumentView, String> {
    let collection = crate::mongo::database().collection::<Document>(&schema.key);
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let fields = validate_document_fields(&schema.fields, payload.fields, true)?;
    let search_text = build_search_text(&schema.fields, &fields);

    let mut document = doc! {
        "_id": ObjectId::new(),
        "schema_key": &schema.key,
        "parent_id": payload.parent_id,
        "status": payload.status.unwrap_or_else(|| "draft".to_string()),
        "enabled": payload.enabled.unwrap_or(true),
        "created_at": now,
        "updated_at": now,
        "created_by": actor.as_ref().map(|value| value.username.clone()),
        "updated_by": actor.as_ref().map(|value| value.username.clone()),
        "created_by_user_id": actor.as_ref().map(|value| value.user_id),
        "updated_by_user_id": actor.as_ref().map(|value| value.user_id),
        "fields": json_object_to_bson_document(&fields)?,
        "search_text": search_text,
    };

    collection
        .insert_one(&document)
        .await
        .map_err(map_mongo_error)?;
    map_document_view(&schema.key, std::mem::take(&mut document))
}

fn empty_object() -> Value {
    Value::Object(serde_json::Map::new())
}

fn inject_locale_field(value: Value, locale_field: &str, locale: String) -> Result<Value, String> {
    let mut map = value
        .as_object()
        .cloned()
        .ok_or_else(|| "document_fields_must_be_object".to_string())?;
    let normalized_locale = locale.trim().to_ascii_lowercase();
    if normalized_locale.is_empty() {
        return Err("locale_required".to_string());
    }
    map.insert(
        locale_field.to_string(),
        Value::String(normalized_locale.replace('-', "_")),
    );
    Ok(Value::Object(map))
}

async fn find_localized_document(
    translation_schema: &SchemaRecord,
    root_document_id: &str,
    locale: &str,
) -> Result<Option<DocumentView>, String> {
    let Some(i18n) = &translation_schema.i18n else {
        return Ok(None);
    };
    let Some(locale_field) = &i18n.locale_field else {
        return Ok(None);
    };
    let collection = crate::mongo::database().collection::<Document>(&translation_schema.key);
    let filter = doc! {
        "schema_key": &translation_schema.key,
        "parent_id": root_document_id,
        format!("fields.{locale_field}"): locale,
    };
    let Some(document) = collection.find_one(filter).await.map_err(map_mongo_error)? else {
        return Ok(None);
    };
    map_document_view(&translation_schema.key, document).map(Some)
}

async fn list_available_locales(
    translation_schema: &SchemaRecord,
    root_document_id: &str,
) -> Result<Vec<String>, String> {
    let Some(i18n) = &translation_schema.i18n else {
        return Ok(Vec::new());
    };
    let Some(locale_field) = &i18n.locale_field else {
        return Ok(Vec::new());
    };
    let collection = crate::mongo::database().collection::<Document>(&translation_schema.key);
    let cursor = collection
        .find(doc! {
            "schema_key": &translation_schema.key,
            "parent_id": root_document_id,
        })
        .await
        .map_err(map_mongo_error)?;
    let documents: Vec<Document> = cursor.try_collect().await.map_err(map_mongo_error)?;

    let mut locales = documents
        .into_iter()
        .filter_map(|document| {
            document
                .get_document("fields")
                .ok()
                .and_then(|fields| fields.get_str(locale_field).ok())
                .map(str::to_string)
        })
        .collect::<Vec<_>>();
    locales.sort();
    locales.dedup();
    Ok(locales)
}

fn ensure_field_type(definition: &SchemaFieldDefinition, value: &Value) -> Result<(), String> {
    let valid = match definition.field_type {
        SchemaFieldType::String => value.is_string(),
        SchemaFieldType::Integer => value.as_i64().is_some(),
        SchemaFieldType::Float => value.as_f64().is_some(),
        SchemaFieldType::Boolean => value.is_boolean(),
        SchemaFieldType::Object => value.is_object(),
        SchemaFieldType::Array => value.is_array(),
    };
    if !valid {
        return Err(format!("field_type_invalid:{}", definition.key));
    }

    if let Some(options) = &definition.options
        && let Some(string_value) = value.as_str()
        && !options.iter().any(|option| option == string_value)
    {
        return Err(format!("field_option_invalid:{}", definition.key));
    }

    Ok(())
}

fn json_object_to_bson_document(value: &Value) -> Result<Document, String> {
    match bson::to_bson(value).map_err(|error| error.to_string())? {
        Bson::Document(document) => Ok(document),
        _ => Err("document_fields_must_be_object".to_string()),
    }
}

fn bson_document_to_json_value(document: Document) -> Result<Value, String> {
    serde_json::to_value(document).map_err(|error| error.to_string())
}

fn merge_json_objects(left: Value, right: Value) -> Result<Value, String> {
    let mut left = left
        .as_object()
        .cloned()
        .ok_or_else(|| "document_fields_must_be_object".to_string())?;
    let right = right
        .as_object()
        .cloned()
        .ok_or_else(|| "document_fields_must_be_object".to_string())?;
    for (key, value) in right {
        left.insert(key, value);
    }
    Ok(Value::Object(left))
}

fn build_search_text(definitions: &[SchemaFieldDefinition], fields: &Value) -> String {
    let Some(map) = fields.as_object() else {
        return String::new();
    };

    definitions
        .iter()
        .filter(|definition| definition.searchable)
        .filter_map(|definition| map.get(&definition.key))
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_object_id(value: &str) -> Result<ObjectId, String> {
    let normalized = value.trim().split('?').next().unwrap_or(value).trim();
    if let Some(prefix) = normalized.get(..24)
        && prefix.chars().all(|ch| ch.is_ascii_hexdigit())
    {
        return ObjectId::parse_str(prefix).map_err(|_| "invalid_id".to_string());
    }
    ObjectId::parse_str(normalized).map_err(|_| "invalid_id".to_string())
}

fn map_mongo_error(error: MongoError) -> String {
    error.to_string()
}
