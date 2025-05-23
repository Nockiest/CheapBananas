-- Migration to replace shop_id with shop_name

-- Step 1: Add the new shop_name column
ALTER TABLE product_entries ADD COLUMN shop_name TEXT;

-- Step 2: Populate shop_name with corresponding shop names
UPDATE product_entries
SET shop_name = shops.name
FROM shops
WHERE product_entries.shop_id = shops.id;

-- Step 3: Drop the shop_id column
ALTER TABLE product_entries DROP COLUMN shop_id;

-- Step 4: Add a NOT NULL constraint to shop_name if required
ALTER TABLE product_entries ALTER COLUMN shop_name SET NOT NULL;
