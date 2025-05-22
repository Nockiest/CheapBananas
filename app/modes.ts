const MODES = [
	{
		key: 'productEntry',
		label: 'Product Entry',
		fields: [
			{ label: 'Product Name', required: true, suggestions: ['banana', 'bread', 'butter', 'beans'] },
			{ label: 'Price', required: true, suggestions: [] },
			{ label: 'Product Volume', required: false, suggestions: [] },
			{ label: 'Unit', required: true, suggestions: ['kg', 'l', 'ks'] },
			{ label: 'Shop Name', required: true, suggestions: ['tesco', 'lidl', 'albert', 'billa'] }, // Make Shop Name required
			{ label: 'Tags', required: false, suggestions: [] },
			{ label: 'Notes', required: false, suggestions: [] },
			{ label: 'Date', required: false, suggestions: [] },
		],
	},
	{
		key: 'product',
		label: 'Product',
		fields: [
			{ label: 'Name', required: true, suggestions: ['banana', 'bread', 'butter', 'beans'] },
			{ label: 'Notes', required: false, suggestions: [] },
			{ label: 'Tags', required: false, suggestions: [] },
		],
	},
	{
		key: 'shop',
		label: 'Shop',
		fields: [
			{ label: 'Name', required: true, suggestions: ['tesco', 'lidl', 'albert', 'billa'] },
			{ label: 'Notes', required: false, suggestions: [] },
		],
	},
];
export default  MODES
