// @generated automatically by Diesel CLI.

diesel::table! {
    agents (id) {
        #[max_length = 100]
        id -> Varchar,
        #[max_length = 200]
        name -> Varchar,
        description -> Nullable<Text>,
        #[max_length = 100]
        model -> Varchar,
        preamble -> Text,
        temperature -> Float4,
        top_k_documents -> Int4,
        tools -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    brochures (id) {
        id -> Uuid,
        #[max_length = 500]
        title -> Varchar,
        description -> Text,
        #[max_length = 500]
        file_name -> Varchar,
        file_url -> Text,
        #[max_length = 50]
        file_type -> Varchar,
        file_size_bytes -> Int8,
        product_ids -> Jsonb,
        #[max_length = 200]
        category -> Varchar,
        #[max_length = 10]
        language -> Varchar,
        is_public -> Bool,
        download_count -> Int8,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    company_info (id) {
        id -> Uuid,
        #[max_length = 100]
        key -> Varchar,
        value -> Text,
        #[max_length = 100]
        category -> Varchar,
        #[max_length = 10]
        language -> Varchar,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    conversations (id) {
        id -> Uuid,
        #[max_length = 100]
        agent_id -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    messages (id) {
        id -> Uuid,
        conversation_id -> Uuid,
        #[max_length = 20]
        role -> Varchar,
        content -> Text,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        #[max_length = 20]
        reaction_type -> Nullable<Varchar>,
        #[max_length = 255]
        reaction_user_id -> Nullable<Varchar>,
        reaction_feedback -> Nullable<Text>,
        reacted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    document_chunks (id) {
        id -> Uuid,
        document_id -> Uuid,
        content -> Text,
        chunk_index -> Int4,
        metadata -> Jsonb,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    documents (id) {
        id -> Uuid,
        #[max_length = 500]
        title -> Varchar,
        content -> Text,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    faqs (id) {
        id -> Uuid,
        question -> Text,
        answer -> Text,
        #[max_length = 200]
        category -> Varchar,
        #[max_length = 10]
        language -> Varchar,
        is_active -> Bool,
        view_count -> Int8,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    jobs (id) {
        id -> Uuid,
        #[max_length = 100]
        job_type -> Varchar,
        payload -> Jsonb,
        #[max_length = 50]
        status -> Varchar,
        result -> Nullable<Jsonb>,
        error -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    product_brochures (product_id, brochure_id) {
        product_id -> Uuid,
        brochure_id -> Uuid,
    }
}

diesel::table! {
    products (id) {
        id -> Uuid,
        #[max_length = 500]
        name -> Varchar,
        description -> Text,
        #[max_length = 200]
        category -> Varchar,
        price -> Nullable<Numeric>,
        #[max_length = 10]
        currency -> Nullable<Varchar>,
        features -> Jsonb,
        specifications -> Jsonb,
        image_urls -> Jsonb,
        is_active -> Bool,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(document_chunks -> documents (document_id));
diesel::joinable!(messages -> conversations (conversation_id));
diesel::joinable!(product_brochures -> brochures (brochure_id));
diesel::joinable!(product_brochures -> products (product_id));

diesel::allow_tables_to_appear_in_same_query!(
    agents,
    brochures,
    company_info,
    conversations,
    document_chunks,
    documents,
    faqs,
    jobs,
    messages,
    product_brochures,
    products,
);
