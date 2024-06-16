insert into entries (product, amount, unit, note, user_id, group_id) values
-- Alice' added entries
('Apples', 6, 'pieces', 'Any sort is ok.', 1, 1),
('Bread', 1, 'piece', 'Wholegrain', 1, null),
('Water', 3, 'l', null, 1, null),
-- Bob's added entries
('Bananas', 6, 'pieces', 'I want to bake banana bread, so overripe ones are ok', 2, 1),
('Chicken breast', 500, 'g', null, 2, null),
('Water', 3, 'l', 'Carbonated', 2, null),
-- Bob also added water to the MegaTech Corporation list
('Water', 30, 'l', 'Carbonated', 2, 2),
('Water', 30, 'l', 'Regular', 2, 2)