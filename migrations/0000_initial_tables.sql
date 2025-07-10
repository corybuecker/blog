create table pages (
    id serial primary key not null,
    content text not null,
    created_at timestamp with time zone not null,
    description text not null,
    markdown text not null,
    preview text not null,
    published_at timestamp with time zone,
    revised_at timestamp with time zone,
    slug text not null,
    title text not null,
    updated_at timestamp with time zone not null
);

create table users (
    id serial primary key not null,
    email text not null
);
