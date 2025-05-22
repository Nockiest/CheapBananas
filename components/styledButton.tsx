import React from 'react';

interface StyledButtonProps {
  onClick: () => void;
  disabled?: boolean;
  children: React.ReactNode;
  isActive?: boolean;
}

const StyledButton: React.FC<StyledButtonProps> = ({ onClick, disabled = false, children, isActive = false }) => {
  const baseStyle: React.CSSProperties = {
    marginRight: 8,
    padding: '10px 24px',
    fontSize: 16,
    borderRadius: 8,
    border: '1px solid #ccc',
    background: disabled ? '#ccc' : isActive ? '#e6f0ff' : '#fff',
    color: disabled ? '#666' : isActive ? '#007aff' : '#222',
    fontWeight: isActive ? 'bold' : 'normal',
    cursor: disabled ? 'not-allowed' : 'pointer',
    transition: 'all 0.2s ease-in-out',
  };

  return (
    <button className={children?.toString()} onClick={onClick} disabled={disabled} style={baseStyle}>
      {children}
    </button>
  );
};

export default StyledButton;