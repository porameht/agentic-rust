-- Products and Brochures schema for Sales Agent

-- Products table
CREATE TABLE IF NOT EXISTS products (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    category VARCHAR(200) NOT NULL,
    price DECIMAL(15, 2),
    currency VARCHAR(10),
    features JSONB NOT NULL DEFAULT '[]',
    specifications JSONB NOT NULL DEFAULT '{}',
    image_urls JSONB NOT NULL DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes on products
CREATE INDEX IF NOT EXISTS idx_products_category ON products(category);
CREATE INDEX IF NOT EXISTS idx_products_is_active ON products(is_active);
CREATE INDEX IF NOT EXISTS idx_products_price ON products(price);
CREATE INDEX IF NOT EXISTS idx_products_name ON products USING gin(to_tsvector('english', name));
CREATE INDEX IF NOT EXISTS idx_products_description ON products USING gin(to_tsvector('english', description));
CREATE INDEX IF NOT EXISTS idx_products_features ON products USING gin(features);
CREATE INDEX IF NOT EXISTS idx_products_metadata ON products USING gin(metadata);

-- Brochures table
CREATE TABLE IF NOT EXISTS brochures (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    file_name VARCHAR(500) NOT NULL,
    file_url TEXT NOT NULL,
    file_type VARCHAR(50) NOT NULL,
    file_size_bytes BIGINT NOT NULL DEFAULT 0,
    product_ids JSONB NOT NULL DEFAULT '[]',
    category VARCHAR(200) NOT NULL DEFAULT '',
    language VARCHAR(10) NOT NULL DEFAULT 'th',
    is_public BOOLEAN NOT NULL DEFAULT true,
    download_count BIGINT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes on brochures
CREATE INDEX IF NOT EXISTS idx_brochures_category ON brochures(category);
CREATE INDEX IF NOT EXISTS idx_brochures_file_type ON brochures(file_type);
CREATE INDEX IF NOT EXISTS idx_brochures_language ON brochures(language);
CREATE INDEX IF NOT EXISTS idx_brochures_is_public ON brochures(is_public);
CREATE INDEX IF NOT EXISTS idx_brochures_product_ids ON brochures USING gin(product_ids);
CREATE INDEX IF NOT EXISTS idx_brochures_title ON brochures USING gin(to_tsvector('english', title));

-- Company info table
CREATE TABLE IF NOT EXISTS company_info (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key VARCHAR(100) UNIQUE NOT NULL,
    value TEXT NOT NULL,
    category VARCHAR(100) NOT NULL DEFAULT 'general',
    language VARCHAR(10) NOT NULL DEFAULT 'th',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_company_info_key ON company_info(key);
CREATE INDEX IF NOT EXISTS idx_company_info_category ON company_info(category);

-- FAQ table
CREATE TABLE IF NOT EXISTS faqs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    category VARCHAR(200) NOT NULL DEFAULT 'general',
    language VARCHAR(10) NOT NULL DEFAULT 'th',
    is_active BOOLEAN NOT NULL DEFAULT true,
    view_count BIGINT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_faqs_category ON faqs(category);
CREATE INDEX IF NOT EXISTS idx_faqs_language ON faqs(language);
CREATE INDEX IF NOT EXISTS idx_faqs_is_active ON faqs(is_active);
CREATE INDEX IF NOT EXISTS idx_faqs_question ON faqs USING gin(to_tsvector('english', question));

-- Product-brochure relationship table
CREATE TABLE IF NOT EXISTS product_brochures (
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    brochure_id UUID NOT NULL REFERENCES brochures(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, brochure_id)
);

-- Triggers for updated_at
CREATE OR REPLACE TRIGGER update_products_updated_at
    BEFORE UPDATE ON products
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE OR REPLACE TRIGGER update_brochures_updated_at
    BEFORE UPDATE ON brochures
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE OR REPLACE TRIGGER update_company_info_updated_at
    BEFORE UPDATE ON company_info
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE OR REPLACE TRIGGER update_faqs_updated_at
    BEFORE UPDATE ON faqs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Insert default Sales Agent
INSERT INTO agents (id, name, description, model, preamble, temperature, top_k_documents, tools)
VALUES (
    'sales-agent',
    'Sales Agent',
    'AI assistant for sales support and product recommendations',
    'gpt-4',
    'คุณเป็นผู้ช่วยฝ่ายขายและบริการลูกค้าของบริษัท',
    0.7,
    5,
    '["product_search", "get_brochure", "company_info"]'
) ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    tools = EXCLUDED.tools;
