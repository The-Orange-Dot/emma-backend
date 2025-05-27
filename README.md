# Introduction

This is a personal project aimed towards ecommerce and increasing user engagement through AI vision models, text generation and retrieval-augmented generation (RAG). The purpose of this project is to provide the LLM with specific context to provide users with better information.

In this case, the model that we use will be able to use a database of products to suggest users what products are the best for their problem/question. Here is the tech stack and the reasoning for it:

**PostgreSQL**: Honestly, it's what I'm most comfortable with. For me, this database offers the ability for larger scaling with advanced queries.

**PGAI**: I wanted the ability to learn and implement RAG to help suggest products without the products becoming "outdated." PGAI offers extensions in the PostgreSQL database that allows chunking data, vectorizing the data, and embedding the data into the generated text for the LLM.

**Rust**: This is something that I'm fairly new in. But I chose to use Rust for pure speed and resource safety. I'm not the best developer, but my previous experience with Node.js has been palatable at best. Rust yells at me a lot, so I know I'm on the right path working with it.

```
[dependencies]
actix = "0.13.5"
dotenv = "0.15.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0.140"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls", "macros", "chrono"] }
error = "0.1.9"
features = "0.10.0"
futures = "0.3.31"
validator = { version = "0.16", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
uuid = "1.16.0"
actix-web = "4.11.0"
reqwest = { version = "^0.11", features = ["blocking", "json"]}
base64 = "0.22.1"
image = "0.25.6"
backoff = { version = "0.4", features = ["tokio"] }
chrono = "0.4.41"
regex = "1.10"
```

## Things to do in this order

- Set up Ollama on a remote server with a GPU
- Set up PostgreSQL database through docker with PGAI

# Ollama on remote/local server

## SETUP

If you are setting up the PostgreSQL database on the same system as your database, you can skip this. This will be the workhorse that handles all the AI and text generation.

Install Ollama for your system (Windows, Linux, or even Mac?)

Fetch the model through `$ ollama pull [Model]`

## MODELS

For this app I am currently using gemma3:12b and nomic-embed-text. I'm hoping to move up to Gemma3:27b with better hardware eventually.

**Gemma3**: A vision model that was developed by Google to be performant and small.

**Nomic Embed Text**: An embedding model for Vectorizing data to put into the PostgreSQL database.

Once you have the models and and run `$ ollama serve`, you should be all good on this end. The IP should just be http://server-ip:11434.

# Setting up the Postgres Database

## CREATING DOCKER COMPOSE FILE

The Docker compose file will have:

- PostgreSQL database with dependencies and env variables set
- Vectorized Worker
- Tailscale

```
version: '3.8'
name: pgai
services:
  tailscale:
    image: tailscale/tailscale:latest
    container_name: tailscale
    hostname: pgai-db
    environment:
      - TS_AUTHKEY=tskey-auth-notarealauthkey-itjustlookslikeoneasanexample
      - TS_STATE_DIR=/var/lib/tailscale
      - TS_EXTRA_ARGS=--advertise-exit-node
    volumes:
      - tailscale-data:/var/lib/tailscale
      - /dev/net/tun:/dev/net/tun
    cap_add:
      - NET_ADMIN
      - NET_RAW
    restart: unless-stopped
    network_mode: "host"

  db:
    image: timescale/timescaledb-ha:pg17
    environment:
      POSTGRES_PASSWORD: postgres
      PGAI_OLLAMA_HOST: "http://123.45.67.89:11434"
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/home/postgres/pgdata/data
    depends_on:
      - tailscale
    command: [
      "-c", "ai.ollama_host=http://123.45.67.89:11434",
    ]

  vectorizer-worker:
    image: timescale/pgai-vectorizer-worker:latest
    environment:
      PGAI_VECTORIZER_WORKER_DB_URL: postgres://postgres:postgres@987.65.43.21:5432/postgres
      OLLAMA_HOST: http://123.45.67.89:11434 -- Notice the difference between the postgres and ollama ip
    command: [ "--poll-interval", "5s" ]

  ollama:
    image: ollama/ollama
    ports:
      - "11434:11434"
    volumes:
      - ollama_data:/root/.ollama

volumes:
  pgdata:
  tailscale-data:
  ollama_data:
```

Save as "docker-compose.yaml" and put it in a folder

### Setting up your Ollama model for tone of voice

You'll probably want to embed a system message for the model you're working with. I prefer to add the system message to the model itself so it adds a bit of flexibility on calling models on the fly. My models are being hosted on a remote Windows PC with a RTX 4080 right now.

### Create a Model File

```
wsl nano new_model_name.txt

```

### Add the system message

```
FROM model:params

SYSTEM "System message here!"

```

Example:

```
FROM Gemma3:12b

SYSTEM "You're an editor who works with gurus and specialists who sends messages the every day end user. You specialize in being a ghost editor who never ever changes the content or tone of the text given to you, but places products organically into the text itself to suggest to the end user."

```

```
FROM Gemma3:12b

SYSTEM "You are a beauty and skincare guru who focuses on enhancing skin and promoting skincare routines to clients directly. You should speak in a positive calm tone of voice. You like to promote products that people can use to enhance their skin but you always focus on natural beauty. Your job is to look at images and answer skincare questions from clients directly based on the image that you see. You must NEVER give medical advice to the clients."

```

### This will create a duplicate of the model with an embedded system prompt

You'll probably need to have Ollama running in the background with `ollama serve` or use 2 terminals to create it.

```
ollama create new_model:blah -f ./path/to/new_model_name.txt
```

Example:

```
ollama create Gemma3:12b-product-placement -f ./product_editor_model.txt
```

## CREATE TABLE WITH EMBEDDING VECTORIZED COLUMN

This command should be run in docker to trigger the PGAI extension. Otherwise, you'll get an error that says vector(768) is unknown. **YOU MUST KEEP THE EMBEDDING COLUMN IN THERE!!**

```
sudo docker exec -it pgai-db-1 psql -c "CREATE TABLE public.products
( id BIGSERIAL PRIMARY KEY,
  row1 TEXT
  row2 INT
  row3 JSONB
  ...
  embedding vector(768)
);"
```

Example:

```
sudo docker exec -it pgai-db-1 psql -c "
DROP TABLE public.products;
CREATE TABLE public.products
( id BIGSERIAL PRIMARY KEY,
  name TEXT,
  category TEXT DEFAULT '',
  handle TEXT DEFAULT '',
  type TEXT DEFAULT '',
  description TEXT DEFAULT '',
  vendor TEXT,
  price TEXT,
  tags TEXT DEFAULT '',
  seo_title TEXT DEFAULT '',
  seo_description TEXT DEFAULT '',
  published TEXT,
  status TEXT DEFAULT 'draft',
  image TEXT,
  image_position TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  embedding_updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  embedding vector(768)
);"
```

Then run this command to remove the NOT NULL default on created_at and updated_at

```
ALTER TABLE public.products ALTER COLUMN updated_at DROP NOT NULL;
ALTER TABLE public.products ALTER COLUMN created_at DROP NOT NULL;
```

## COPYING CSV TABLE OVER TO POSTGRES DB

**This section is only temporary. This app will eventually pull from a products API (on shiopify for example) and duplicate the database that it can modify and embed.**

If you're adding data from a .csv file to your database in Docker, copy the files into the local system that has the database and run the command below. Just make sure the values (name, categories, etc.) are the same in your database and the command below. Also note the "HEADER" flag below, which means the columns in the .csv should have the same format (capitalization matters).

If your database is in a virtual machine, you need to send the file to the virtual machine first before you can send it into the Docker container. (There's probably an easier way to do this, but...whatever)

### 1: Sending local .csv to VM

```
scp /path/to/local/file.csv username@vm_ip_address:/path/to/destination/
```

Example:

```
scp ./Downloads/products.csv Thomas@123.45.67.89:/tmp/products.csv
```

### 2: Sending .csv from VM to Docker container

```
docker cp /path/to/your/file.csv container_name_or_id:/destination/path/

```

Example:

```
docker cp /tmp/products.csv pgai-db-1:/tmp/products.csv
```

### 3: Copy data from Docker .csv to PostgreSQL database

```
sudo docker exec -it pgai-db-1 psql -c "\copy table_name (row1, row2, row3, row4, etc.) from /path/to/data.csv WITH CSV HEADER DELIMITER ','"
```

Example:

```
sudo docker exec -it pgai-db-1 psql -c "\copy products (handle, name, body, category, type, tags, published, price, image, set_title, seo_description, status) from /tmp/products.csv WITH CSV HEADER DELIMITER ','"
```

If you have variants in your Shopify .csv, you can use this to remove variants from your .csv to upload

```
docker exec pgai-db-1 bash -c "
    # Find which column contains 'name' in the header
    name_col=\$(awk -F, '
        NR==1 {
            for (i=1; i<=NF; i++) {
                if (\$i == \"name\") {
                    print i;
                    exit;
                }
            }
        }
    ' /tmp/shiko_test_products.csv)

    # Filter rows where name column is empty/NULL
    awk -F, -v col=\"\$name_col\" '
    NR==1 { print; next }  # Keep header
    {
        if (\$col != \"\" && \$col != \"NULL\") {
            print
        }
    }
    ' /tmp/shiko_test_products.csv > /tmp/clean_data.csv &&

    # Import only clean rows
    psql -c \"\\copy products (handle, name, body, category, type, tags, published, price, image, seo_title, seo_description, status)
            FROM '/tmp/clean_data.csv'
            WITH CSV HEADER DELIMITER ','\"
"
```

## EMBEDDING DATA TO EMBEDDING COLUMN

Again, make sure the values are the correct values. We're using the model nomic-embed-text to vectorize the data into an "embedding" column for the data to be used later for context.

(Use '$s' to place data columns into the embedding column)

```
UPDATE public.products SET embedding = ai.ollama_embed('nomic-embed-text', format('Hello World! This is $s speaking from $s!', name, location));
```

Example:

```
 UPDATE public.products SET embedding = ai.ollama_embed('nomic-embed-text', format('NAME: %s - PRICE: %s - DESCRIPTION: %s %s %s %s - CATEGORY: %s STATUS: %s %s - ', name, price, body, tags, seo_description, seo_title, category, published, status));
```

MEMOS:

USE THIS TO CREATE, COPY, AND RUN THE EMBED

```
DROP TABLE IF EXISTS public.temp_table;

CREATE TABLE public.temp_table
(
  handle TEXT,
  name TEXT,
  description TEXT,
  vendor TEXT,
  category TEXT,
  type TEXT,
  tags TEXT,
  published TEXT,
  price TEXT,
  image TEXT,
  image_position TEXT,
  seo_title TEXT,
  seo_description TEXT,
  status TEXT
  );

 COPY public.temp_table FROM '/tmp/shiko_test_products.csv' DELIMITER ',' CSV HEADER;

 INSERT INTO public.products (handle, name, description, vendor, category, type, tags, published, price, image, image_position, seo_title, seo_description, status)
 SELECT handle, name, description, vendor, category, type, tags, published, price, image, image_position, seo_title, seo_description, status
 FROM temp_table
 WHERE name IS NOT NULL AND published != 'FALSE'
 ;

 DROP TABLE public.temp_table;

 UPDATE public.products SET embedding = ai.ollama_embed('nomic-embed-text', format('NAME: %s - PRICE: %s - DESCRIPTION: %s %s %s %s - CATEGORY: %s STATUS: %s %s - ', name, price, description, tags, seo_description, seo_title, category, published, status));

 SELECT * from public.products;
```

## This will change the column names if you need.

```
sudo docker exec -i pgai-db-1 bash -c '
  sed -i "1s/old_col_name/new_col_name/" /tmp/shiko_test_products.csv
'
```

# Authentication for shops

CREATE TABLE public.temp_table
(
id uuid DEFAULT gen_random_uuid(),
created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
subcription_ends TIMESTAMP
name VARCHAR,
domain VARCHAR,
platform VARCHAR,
status VARCHAR,
plan VARCHAR,
sys_prompt VARCHAR,
db_table VARCHAR,
db VARCHAR,
db_pw VARCHAR,
email VARCHAR
);
