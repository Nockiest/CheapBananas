export enum Unit {
  Ks = "ks",
  Kg = "kg",
  L = "l",
}

export interface Product {
  id: string;
  name: string;
  notes?: string;
  tags?: string[];
}

export interface ProductEntry {
  id: string;
  product_id: string;
  price: number;
  product_volume?: number;
  unit: Unit;
  shop_name?: string;
  date?: string; // ISO 8601 format
  notes?: string;
  quantity?: number;
}

export interface Shop {
  id: string;
  name: string;
  notes?: string;
}

export interface ProductFilter {
  id?: string;
  name?: string;
  notes?: string;
  tag?: string;
  product_id?: string;
  min_price?: number;
  max_price?: number;
  product_volume?: number;
  unit?: string;
  shop_id?: string;
  date?: string; // ISO 8601 format
}

export interface ShopFilter {
  id?: string;
  name?: string;
  notes?: string;
}

export type ProductEntryFilter = ProductFilter;