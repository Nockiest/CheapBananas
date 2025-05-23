'use client';
import { useState, useEffect } from 'react';
import axios from 'axios';
import { ProductEntry } from '@/types/types';
import Link from 'next/link';
const ProductEntriesPage = () => {
  const [productName, setProductName] = useState('');
  const [productEntries, setProductEntries] = useState<ProductEntry[]>([]);
  const [error, setError] = useState('');
  const [productSuggestions, setProductSuggestions] = useState<string[]>([]);

  const fetchProductEntries = async () => {
    try {
      setError('');
      const productResponse = await axios.get(
        `http://localhost:4000/products/filter?name=${encodeURIComponent(productName)}`
      );

      if (productResponse.data.length === 0) {
        setError('No product found with the given name.');
        return;
      }

      const productId = productResponse.data[0].id;
      const entriesResponse = await axios.get(
        `http://localhost:4000/product-entries/filter?product_id=${productId}`
      );

      const sortedEntries = entriesResponse.data.sort((a:ProductEntry, b:ProductEntry) => (a.price/a.product_volume) - (b.price/b.product_volume));
      setProductEntries(sortedEntries);
    } catch (err) {
      setError('Failed to fetch product entries. Please try again later.' + (err as Error).message);
    }
  };

  const deleteProductEntry = async (id: string) => {
    try {
      await axios.delete(`http://localhost:4000/product-entries/${id}`);
      setProductEntries((prevEntries) => prevEntries.filter((entry) => entry.id !== id));
    } catch (err) {
      setError('Failed to delete product entry. Please try again later.');
    }
  };

  const deleteProductAndEntries = async () => {
    if (!confirm('Are you sure you want to delete this product and all its entries?')) {
      return;
    }

    try {
      setError('');
      const productResponse = await axios.get(
        `http://localhost:4000/products/filter?name=${encodeURIComponent(productName)}`
      );

      if (productResponse.data.length === 0) {
        setError('No product found with the given name.');
        return;
      }

      const productId = productResponse.data[0].id;
      await axios.delete(`http://localhost:4000/products/${productId}`);

      setProductEntries([]);
      setProductName('');
      alert('Product and its entries have been deleted successfully.');
    } catch (err) {
      setError('Failed to delete product and its entries. Please try again later.');
    }
  };

  const fetchProductSuggestions = async (query: string) => {
    if (!query) {
      setProductSuggestions([]);
      return;
    }

    try {
      const response = await axios.get(
        `http://localhost:4000/products/filter?name=${encodeURIComponent(query)}`
      );
      setProductSuggestions(response.data.map((product: { name: string }) => product.name));
    } catch (err) {
      console.error('Failed to fetch product suggestions:', err);
    }
  };

  useEffect(() => {
    const delayDebounceFn = setTimeout(() => {
      fetchProductSuggestions(productName);
    }, 300);

    return () => clearTimeout(delayDebounceFn);
  }, [productName]);

  return (
    <div style={{ padding: '20px' }}>
      <Link href='/' style={{ textDecoration: 'none', color: 'blue', paddingBottom: '10px', display: 'block' }}>Go Back</Link>
      <h1 style={{ paddingBottom: '20px' }}>Product Entries</h1>
      <div style={{ paddingBottom: '20px' }}>
        <input
          type="text"
          placeholder="Enter product name"
          value={productName}
          onChange={(e) => setProductName(e.target.value)}
          style={{ marginRight: '10px', padding: '5px' }}
        />
        {productSuggestions.length > 0 && (
          <ul style={{ border: '1px solid #ccc', marginTop: '5px', padding: '5px', listStyle: 'none' }}>
            {productSuggestions.map((suggestion, index) => (
              <li
                key={index}
                onClick={() => setProductName(suggestion)}
                style={{ cursor: 'pointer', padding: '5px' }}
              >
                {suggestion}
              </li>
            ))}
          </ul>
        )}
        <button onClick={fetchProductEntries} style={{ cursor: 'pointer', padding: '5px 10px' }}>
          Search
        </button>
      </div>
      {error && <p style={{ color: 'red', paddingBottom: '20px' }}>{error}</p>}
      {productEntries.length > 0 && (
        <table style={{ marginTop: '20px', width: '100%', textAlign: 'left', paddingBottom: '20px' }}>
          <thead>
            <tr>
              <th>Shop</th>
              <th>CZK</th>
              <th>Notes</th>
              <th>Volume</th>
              <th>$ per Unit</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            {productEntries.map((entry) => (
              <tr key={entry.id}>
                <td>{entry.shop_name}</td>
                <td>{entry.price}</td>
                <td>{entry.notes || 'N/A'}</td>
                <td>{entry.product_volume ? `${entry.product_volume}${entry.unit}` : 'N/A'}</td>
                <td>{(entry.price / (entry.product_volume || 1)).toFixed(2)}</td>
                <td>
                  <button onClick={() => deleteProductEntry(entry.id)} style={{  cursor: 'pointer', padding: '5px 10px', color: 'red' }}>
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      <button onClick={deleteProductAndEntries} style={{ cursor: 'pointer', padding: '5px 10px', color: 'red', marginLeft: '10px' }}>
        Delete Product and Entries
      </button>
    </div>
  );
};

export default ProductEntriesPage;
