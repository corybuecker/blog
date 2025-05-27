INSERT INTO users (id, email)
VALUES (DEFAULT, 'ios@corybuecker.com');

INSERT INTO pages (
  id,
  content,
  created_at,
  description,
  markdown,
  preview,
  published_at,
  revised_at,
  slug,
  title,
  updated_at
) VALUES (
  DEFAULT,
  'This is the content of the test post.',
  NOW(),
  'A test post.',
  '<p># Test Post\nThis is a test.</p>',
  'Test post preview.',
  NOW(),
  NOW(),
  'test-post',
  'Test Post',
  NOW()
);

