"use client";
import React, { useState, useEffect, useRef } from 'react';
import styles from './styles';
import { UNIT_CONVERSIONS, normalizeUnit, normalizePricePerPiece } from './unitConversion';
import { v4 as uuidv4 } from 'uuid';

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

export default function HomePage() {
	const [mode, setMode] = useState(MODES[0].key);
	const currentMode = MODES.find(m => m.key === mode)!;
	const [text, setText] = useState('');
	const [success, setSuccess] = useState(false);
	const [sentEntry, setSentEntry] = useState<string[] | null>(null);
	const [suggestions, setSuggestions] = useState(
		currentMode.fields.map(f => f.suggestions)
	);
	const [activeSuggestion, setActiveSuggestion] = useState(Array(currentMode.fields.length).fill(null));
	const abortControllers = useRef(Array(currentMode.fields.length).fill(null));
	const [errorMsg, setErrorMsg] = useState<string | null>(null);

	useEffect(() => {
		setSuggestions(currentMode.fields.map(f => f.suggestions));
		setActiveSuggestion(Array(currentMode.fields.length).fill(null));
		setText('');
		setSuccess(false);
		setSentEntry(null);
	}, [mode]);

	const values = text.trim().length > 0 ? text.split(/\s+/) : [];
	while (values.length < currentMode.fields.length) values.push('');

	const fetchSuggestions = async (idx: number, input: string) => {
		let data: string[] = currentMode.fields[idx].suggestions || [];
		if (!input) {
			setSuggestions(s => {
				const copy = [...s];
				copy[idx] = data;
				return copy;
			});
			setActiveSuggestion(a => {
				const copy = [...a];
				copy[idx] = null;
				return copy;
			});
			return;
		}
		await new Promise(res => setTimeout(res, 150));
		const filtered = data.filter(v => v.toLowerCase().startsWith(input.toLowerCase()));
		setSuggestions(s => {
			const copy = [...s];
			copy[idx] = data;
			return copy;
		});
		setActiveSuggestion(a => {
			const copy = [...a];
			copy[idx] = filtered.length > 0 ? filtered[0] : null;
			return copy;
		});
	};

	const handleTableEdit = (idx: number, newValue: string) => {
		setSuccess(false);
		const sanitizedValue = newValue.replace(/\s+/g, '-').toLowerCase();
		const newValues = [...values];
		let lastNonEmpty = newValues.length - 1;
		while (lastNonEmpty > 0 && newValues[lastNonEmpty] === '') lastNonEmpty--;
		if (idx > lastNonEmpty) {
			for (let i = lastNonEmpty + 1; i < idx; i++) {
				newValues[i] = '_';
			}
		}
		newValues[idx] = sanitizedValue;
		let newLastNonEmpty = newValues.length - 1;
		while (newLastNonEmpty > 0 && newValues[newLastNonEmpty] === '') newLastNonEmpty--;
		setText(newValues.slice(0, newLastNonEmpty + 1).join(' '));
		fetchSuggestions(idx, sanitizedValue);
	};

	useEffect(() => {
		setSuggestions(currentMode.fields.map(f => f.suggestions));
		setActiveSuggestion(Array(currentMode.fields.length).fill(null));
	}, [text, mode]);

	const handleSuggestionClick = (idx: number) => {
		if (!activeSuggestion[idx]) return;
		handleTableEdit(idx, activeSuggestion[idx]);
	};

	const REQUIRED_INDICES = currentMode.fields
		.map((f, idx) => (f.required ? idx : null))
		.filter(idx => idx !== null);
	const allRequiredFilled = REQUIRED_INDICES.every(idx => (values[idx!] && values[idx!] !== ''));

	let errorText = '';
	if (text.trim().length > 0 && !allRequiredFilled) {
		errorText = `Please fill in all required fields for ${currentMode.label}.`;
	}

	const handleSend = async () => {
		try {
			let entryValues = [...values];
			let endpoint = '';
			let body: any = {};
			let baseUrl = 'http://localhost:4000';
			if (mode === 'product') {
				endpoint = '/products';
				// The backend expects a Product struct, which requires an id field.
				// We'll generate a UUID here and send it as part of the product object.
				// You can use the uuid library for this:
				// npm install uuid
				// import { v4 as uuidv4 } from 'uuid';
				// For now, use a simple UUID generator:
				const uuid = URL.createObjectURL(new Blob()).split('/').pop();
				body = {
					id: uuid,
					name: entryValues[0],
					notes: entryValues[1] || undefined,
					tags: entryValues[2] ? entryValues[2].split(',').map((t: string) => t.trim()) : undefined,
				};
			} else if (mode === 'productEntry') {
				const id = uuidv4();
				// Normalize product_volume and unit
				let normalized = { value: entryValues[2], unit: entryValues[3] };
				if (entryValues[2] && entryValues[3]) {
					normalized = normalizeUnit(entryValues[2], entryValues[3]);
				}
				body = {
					id,
					product_name: entryValues[0],
					price: entryValues[1] ? Number(entryValues[1]) : undefined,
					product_volume: normalized.value ? Number(normalized.value) : undefined,
					unit: normalized.unit,
					shop_name: entryValues[4] || undefined, // always send as shop_name
					tags: entryValues[5] ? entryValues[5].split(',').map((t: string) => t.trim()) : undefined,
					notes: entryValues[6] || undefined,
					date: entryValues[7] || undefined,
				};
				endpoint = '/product-entries';
			} else if (mode === 'shop') {
				endpoint = '/shops';
				body = {
					name: entryValues[0],
					notes: entryValues[1] || undefined,
				};
			}
			const res = await fetch(baseUrl + endpoint, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(body),
			});
			if (!res.ok) {
				const errText = await res.text();
				throw new Error(errText || 'Failed to send');
			}
			setSuccess(true);
			setSentEntry(entryValues);
			setText('');
			setErrorMsg(null);
		} catch (e: any) {
			setSuccess(false);
			setSentEntry(null);
			setErrorMsg(e?.message || 'Failed to send');
		}
	};

	return (
		<div style={styles.container}>
			<div style={{ marginBottom: 16 }}>
				{MODES.map(m => (
					<button
						key={m.key}
						onClick={() => { setMode(m.key); setText(''); setSuccess(false); setSentEntry(null); }}
						style={{
							marginRight: 8,
							padding: '6px 16px',
							borderRadius: 6,
							border: m.key === mode ? '2px solid #007aff' : '1px solid #ccc',
							background: m.key === mode ? '#e6f0ff' : '#fff',
							color: '#222',
							fontWeight: m.key === mode ? 'bold' : 'normal',
							cursor: 'pointer',
						}}
					>
						{m.label}
					</button>
				))}
			</div>
			<input
				style={styles.input}
				placeholder={
					'Type: ' + currentMode.fields.map(f => f.label.toLowerCase()).join(' ')
				}
				value={text.toLowerCase()}
				onChange={e => { setText(e.target.value.toLowerCase()); setSuccess(false); }}
				autoCapitalize="none"
				autoCorrect="off"
			/>
			{text.trim().length > 0 && (
				<div style={styles.table}>
					{currentMode.fields.map((field, idx) => (
						<div style={styles.row} key={field.label}>
							<span style={styles.cellLabel}>{field.label}</span>
							<div style={{ flex: 1 }}>
								{field.label === 'Date' ? (
									<input
										type="date"
										style={styles.cellInput}
										value={values[idx] || ''}
										onChange={e => handleTableEdit(idx, e.target.value)}
									/>
								) : (
									<input
										style={styles.cellInput}
										value={values[idx] ? values[idx].toLowerCase() : ''}
										onChange={e => handleTableEdit(idx, e.target.value)}
										autoCapitalize="none"
										autoCorrect="off"
										onFocus={() => fetchSuggestions(idx, values[idx] ? values[idx].toLowerCase() : '')}
										aria-label="Value input"
									/>
								)}
								{activeSuggestion[idx] && values[idx] && activeSuggestion[idx].toLowerCase() !== values[idx].toLowerCase() && (
									<span
										style={styles.suggestion}
										onClick={() => handleSuggestionClick(idx)}
									>
										{activeSuggestion[idx].toLowerCase()}
									</span>
								)}
							</div>
						</div>
					))}
				</div>
			)}
			{errorText !== '' && (
				<div style={styles.errorText}>{errorText}</div>
			)}
			{success && errorText === '' && sentEntry && (
				<div style={styles.successText}>
					{currentMode.label} sent to backend!<br />
					<span style={{ fontSize: 14, color: '#222' }}>
						Sent values: [{sentEntry.map((v, i) => `${currentMode.fields[i]?.label || 'Extra'}: ${v}`).join(', ')}]
					</span>
				</div>
			)}
			{errorMsg && (
				<div style={{ color: 'red', fontSize: 16, margin: '8px 0', textAlign: 'center' }}>{errorMsg}</div>
			)}
			<button
				onClick={handleSend}
				disabled={!allRequiredFilled}
				style={{
					marginTop: 12,
					padding: '10px 24px',
					fontSize: 16,
					borderRadius: 8,
					background: !allRequiredFilled ? '#ccc' : '#007aff',
					color: '#fff',
					border: 'none',
					cursor: !allRequiredFilled ? 'not-allowed' : 'pointer',
				}}
			>
				Send to backend
			</button>
		</div>
	);
//
}