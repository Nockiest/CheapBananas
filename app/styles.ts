// styles.ts
import React from 'react';

const styles: Record<string, React.CSSProperties> = {
  container: {
    minHeight: '100vh',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    background: '#fff',
    padding: '32px 0',
  },
  input: {
    width: '90%',
    maxWidth: 500,
    height: 48,
    border: '1px solid #ccc',
    borderRadius: 8,
    padding: '0 16px',
    fontSize: 18,
    background: '#f9f9f9',
    marginBottom: 24,
    color: '#222',
  },
  table: {
    width: '90%',
    maxWidth: 500,
    borderRadius: 10,
    background: '#f3f3f3',
    padding: 8,
    boxShadow: '0 2px 8px rgba(0,0,0,0.05)',
    marginBottom: 16,
  },
  row: {
    display: 'flex',
    alignItems: 'center',
    marginBottom: 8,
  },
  cellLabel: {
    width: 80,
    fontWeight: 'bold',
    color: '#444',
    fontSize: 16,
  },
  cellInput: {
    flex: 1,
    height: 36,
    border: '1px solid #222222',
    borderRadius: 6,
    padding: '0 10px',
    fontSize: 16,
    background: '#fff',
    color: '#222',
  },
  errorText: {
    color: 'red',
    fontSize: 16,
    margin: '8px 0',
    textAlign: 'center',
  },
  successText: {
    color: 'green',
    fontSize: 16,
    margin: '8px 0',
    textAlign: 'center',
  },
  suggestion: {
    color: '#007aff',
    fontSize: 15,
    marginTop: 2,
    marginLeft: 8,
    cursor: 'pointer',
    display: 'inline-block',
  },
    button: {
    marginRight: 8,
    padding: '6px 16px',
    borderRadius: 6,
    border: '1px solid #ccc',
    background: '#fff',
    color: '#222',
    fontWeight: 'normal',
    cursor: 'pointer',
  },
   buttonActive: {
    border: '2px solid #007aff',
    background: '#e6f0ff',
    fontWeight: 'bold',
    color: '#007aff',
  },
};

// export const buttonStyle = (isActive: boolean): React.CSSProperties => ({
//   marginRight: 8,
//   padding: '6px 16px',
//   borderRadius: 6,
//   border: isActive ? '2px solid #007aff' : '1px solid #ccc',
//   background: isActive ? '#e6f0ff' : '#fff',
//   color: '#222',
//   fontWeight: isActive ? 'bold' : 'normal',
//   cursor: 'pointer',
// });

export default styles;
