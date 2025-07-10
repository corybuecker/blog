---
slug: how-to-make-a-static-blog-in-markdown-and-nextjs
title: How to make a static blog in Markdown and Next.js
description: I had a couple of requirements in mind when starting this site. First, I wanted to use Markdown but build the main content management system (CMS) with React. It also needs to transpile into a static site so that I can host it in a S3 bucket or in Google Cloud Storage. The Markdown content files need to contain front matter that renders to the page.
preview: I had a couple of requirements in mind when starting this site. First, I wanted to use Markdown but build the main content management system (CMS) with React. It also needs to transpile into a static site so that I can host it in a S3 bucket or in Google Cloud Storage. The Markdown content files need to contain front matter that renders to the page.
published_at: 2019-12-01T00:00:00+00:00
revised_at: 2022-01-29T00:00:00+00:00
---

I had a couple of requirements in mind when starting this site. First, I wanted to use Markdown but build the main content management system (CMS) with React. It also needs to transpile into a static site so that I can host it in a S3 bucket or in Google Cloud Storage. The Markdown content files need to contain front matter that renders to the page.

[Next.js](https://nextjs.org/docs) provides these feature and is popular in the React community. There are a couple of extra steps to take when building a static site with transpiled Markdown content.

If you prefer to see the code for this blog, it is [hosted on GitHub](https://github.com/corybuecker/corybuecker.com).

## Setup the project

First, follow the [setup guide](https://nextjs.org/docs) to create a new Next.js application.

Create a `content` folder in your new project and add a sample Markdown file with some front matter.

```yaml
---
title: An introduction
published: 2019-11-30
---

Hi!
```

Install `react-markdown` and `front-matter`. These will be used to convert the Markdown content files into HTML.

```bash
npm install --save-dev react-markdown front-matter
```

## Aggregate the content

Everytime the Next.js development server starts or the site is exported, the content files need to be aggregated and front matter converted into attributes. I found it easiest to convert them into a static JSON file that I imported into the CMS.

I wrote this simple script to convert the content. Please note that this step does not convert the Markdown into HTML. Rather, it converts the front matter into attributes and aggregates all of the content files into an array.

```javascript
const fm = require('front-matter')
const fs = require('fs')
const path = require('path')

const markdownPages = fs.readdirSync(path.join(process.cwd(), 'content'))

console.log(markdownPages)

const compiledPages = []

compiledPages.forEach(pagePath => {
  const contents = fm(
    fs.readFileSync(path.join(process.cwd(), 'content', pagePath), 'utf-8')
  )

  contents.path = pagePath.replace('.md', '')

  compiledPages.push(contents)
})

fs.writeFileSync(
  path.join(process.cwd(), 'src', 'content.json'),
  JSON.stringify(compiledPages.reverse())
)
```

## Next.JS dynamic routing

I needed all of my posts to be accessible on URLs with a slug, e.g. `/posts/2019-12-01-an-introduction`. Next.js provides an mechanism to do this with [dynamic routing](https://nextjs.org/docs#dynamic-routing).

When exporting the static version of this site, Next.js will call `getInitialProps` on all of the exported pages. This is used to pass the dynamic query string to the routed page. This [requires that the function](https://nextjs.org/docs#limitation) extract the correct page from the aggregated content JSON file.

```javascript
import posts from '../src/content.json'

const postsBySlug = posts.reduce((p, post) => {
    p[post.path] = post
    return p
  }, {})
}

export default Post = ({attributes, body, path}) => {
  return (<div>
    <Markdown source={body}></Markdown>
  </div>)
}

Post.getInitialProps = async ({ query }) =>
  postsBySlug[query.slug]
```

The last step is to tell Next.js about the dynamic routes based on the content JSON file. This is accomplished by [creating a `next.config.js` file](https://nextjs.org/docs#custom-configuration).

```javascript
const content = require('./src/content.json')

module.exports = {
  exportPathMap: async function (
    defaultPathMap,
    { dev, dir, outDir, distDir, buildId }
  ) {
    return content.reduce((pages, page) => {
      pages[`/post/${page.path}`] = {
        page: '/post/[slug]',
        query: { slug: page.path }
      }

      return pages
    }, {
      '/index': { page: '/index' },
      '/': { page: '/' }
    })
  }
}
```

At this point, the contents of the `out` directory can be served from a Cloud Storage or S3 bucket.
