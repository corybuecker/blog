--
-- PostgreSQL database dump
--

-- Dumped from database version 17.4 (Debian 17.4-1.pgdg120+2)
-- Dumped by pg_dump version 17.4 (Homebrew)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: pages; Type: TABLE; Schema: public; Owner: blog
--

CREATE TABLE public.pages (
    id integer NOT NULL,
    content text NOT NULL,
    created_at timestamp with time zone NOT NULL,
    description text NOT NULL,
    markdown text NOT NULL,
    preview text NOT NULL,
    published_at timestamp with time zone,
    revised_at timestamp with time zone,
    slug text NOT NULL,
    title text NOT NULL,
    updated_at timestamp with time zone NOT NULL
);


ALTER TABLE public.pages OWNER TO blog;

--
-- Name: pages_id_seq; Type: SEQUENCE; Schema: public; Owner: blog
--

CREATE SEQUENCE public.pages_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.pages_id_seq OWNER TO blog;

--
-- Name: pages_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: blog
--

ALTER SEQUENCE public.pages_id_seq OWNED BY public.pages.id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: blog
--

CREATE TABLE public.users (
    id integer NOT NULL,
    email text NOT NULL
);


ALTER TABLE public.users OWNER TO blog;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: blog
--

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.users_id_seq OWNER TO blog;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: blog
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: pages id; Type: DEFAULT; Schema: public; Owner: blog
--

ALTER TABLE ONLY public.pages ALTER COLUMN id SET DEFAULT nextval('public.pages_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: blog
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: pages pages_pkey; Type: CONSTRAINT; Schema: public; Owner: blog
--

ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_pkey PRIMARY KEY (id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: blog
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: pages_slug_idx; Type: INDEX; Schema: public; Owner: blog
--

CREATE UNIQUE INDEX pages_slug_idx ON public.pages USING btree (slug);


--
-- PostgreSQL database dump complete
--

