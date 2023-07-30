# drop and create the database tailcall_main_db
drop database if exists tailcall_main_db;
create database tailcall_main_db;

# drop and create user tailcall_main_user with default password and all privileges on tailcall_main_db
drop user if exists 'tailcall_main_user'@'localhost';
create user 'tailcall_main_user'@'localhost' identified by 'tailcall';
grant all privileges on tailcall_main_db.* to 'tailcall_main_user'@'localhost';
