pub fn build_shopify_query(limit: usize) -> String {
    format!(r#"
    {{
      products(first: {}) {{
        nodes {{
          id
          title
          tags
          updatedAt
          createdAt
          description
          featuredImage {{
            url
          }}
          seo {{
            description
            title
          }}
          priceRange {{
            minVariantPrice {{
              amount
              currencyCode
            }}
          }}
          vendor
          handle
          publishedAt
          productType
          onlineStoreUrl
          availableForSale
        }}
      }}
    }}
    "#, limit)
}