'use client';
import { useState } from 'react';
import axios from 'axios';
import { ProductEntry } from '@/types/types';
const ProductEntriesPage = () => {
  const [productName, setProductName] = useState('');
 
  const [productEntries, setProductEntries] = useState<ProductEntry[]>([]);
  const [error, setError] = useState('');

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

      const sortedEntries = entriesResponse.data.sort((a:ProductEntry, b:ProductEntry) => a.price - b.price);
      setProductEntries(sortedEntries);
    } catch (err) {
      setError('Failed to fetch product entries. Please try again later.');
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

  return (
    <div style={{ padding: '20px' }}>
      <h1>Product Entries</h1>
      <div>
        <input
          type="text"
          placeholder="Enter product name"
          value={productName}
          onChange={(e) => setProductName(e.target.value)}
          style={{ marginRight: '10px', padding: '5px' }}
        />
        <button onClick={fetchProductEntries} style={{ padding: '5px 10px' }}>
          Search
        </button>
      </div>
      {error && <p style={{ color: 'red' }}>{error}</p>}
      {productEntries.length > 0 && (
        <table   style={{ marginTop: '20px', width: '100%', textAlign: 'left' }}>
          <thead>
            <tr>
              <th>Shop Name</th>
              <th>Price</th>
              <th>Notes</th>
              <th>Quantity</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            {productEntries.map((entry) => (
              <tr key={entry.id}>
                <td>{entry.shop_name}</td>
                <td>{entry.price}</td>
                <td>{entry.notes || 'N/A'}</td>
                <td>{entry.quantity || 'N/A'}</td>
                <td>
                  <button onClick={() => deleteProductEntry(entry.id)} style={{ padding: '5px 10px', color: 'red' }}>
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
};

export default ProductEntriesPage;
