CREATE TABLE IF NOT EXISTS vuelos_entrantes (id int, dest text, llegada timestamp, pos_lat double, pos_lon double, estado text, PRIMARY KEY ((dest), id));
CREATE TABLE IF NOT EXISTS vuelos_salientes (id int, orig text, salida timestamp, pos_lat double, pos_lon double, estado text, PRIMARY KEY ((orig), id));
INSERT INTO vuelos_entrantes (id, dest, llegada, pos_lat, pos_lon, estado) VALUES (1, 'EZE', 564, 5323523.0, 532532.0, 'En vuelo');
INSERT INTO vuelos_entrantes (id, dest, llegada, pos_lat, pos_lon, estado) VALUES (2, 'ROS', 653, 4323523.0, 632532.0, 'En vuelo');
INSERT INTO vuelos_entrantes (id, dest, llegada, pos_lat, pos_lon, estado) VALUES (3, 'URU', 432, 3213521.0, 732234.0, 'En vuelo');
INSERT INTO vuelos_entrantes (id, dest, llegada, pos_lat, pos_lon, estado) VALUES (4, 'BRA', 123, 5433525.0, 465132.0, 'En vuelo');
INSERT INTO vuelos_entrantes (id, dest, llegada, pos_lat, pos_lon, estado) VALUES (5, 'HAW', 456, 3228231.0, 787534.0, 'En vuelo');
INSERT INTO vuelos_entrantes (id, dest, llegada, pos_lat, pos_lon, estado) VALUES (6, 'ORE', 678, 5482625.0, 434512.0, 'En vuelo');