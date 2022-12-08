CREATE TABLE plot_ids (
	user_id BIGINT UNSIGNED NOT NULL,
    plot_inc INT NOT NULL,
    
    PRIMARY KEY (user_id),
    FOREIGN KEY (user_id)
    	REFERENCES users(user_id)
    	ON DELETE CASCADE
);

INSERT INTO plot_ids (user_id, plot_inc)
SELECT user_id, count(user_id) 
FROM plots 
GROUP by user_id;