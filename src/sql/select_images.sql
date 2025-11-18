SELECT id, timestamp, protocol FROM images WHERE timestamp < ? ORDER BY timestamp DESC LIMIT ?;
