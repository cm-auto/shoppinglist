-- uses bcrypt for password hashing
-- cost is DEFAULT_COST from bcrypt rust crate (2024)
insert into users (username, password, display_name) values
('alice', '$2y$12$tsxYQQiWBCTf8cx./l49EuqBDpXGi9uEWZAAPwpsKFyc/UZeXFvIK', 'Alice'), -- password is 'alice'
('bob', '$2y$12$m8k3kGJRBkUZglgxjYwy7ud/psUKUBkmcOBa6R2inJ1DFzlbqeUsC', 'Bob') -- password is 'bob'