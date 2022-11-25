CREATE TABLE IF NOT EXISTS users (
    user_id BIGINT UNSIGNED,
    mc_name VARCHAR(64),

    PRIMARY KEY (user_id)
);

CREATE TABLE IF NOT EXISTS plots (
    plot_id VARCHAR(64),
    user_id BIGINT UNSIGNED,

    PRIMARY KEY (plot_id),
    FOREIGN KEY (user_id)
        REFERENCES users(user_id)
        ON DELETE CASCADE
);