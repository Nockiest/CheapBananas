const fetchSuggestions = async (
    idx: number,
    input: string,
    suggestions: string[][]
): Promise<{ updatedSuggestions: string[][]; activeSuggestion: (string | null)[] }> => {
    let data: string[] = suggestions[idx] || [];
    let updatedSuggestions = [...suggestions];
    let activeSuggestion = Array(suggestions.length).fill(null);

    if (!input) {
        updatedSuggestions[idx] = data;
        return { updatedSuggestions, activeSuggestion };
    }

    await new Promise(res => setTimeout(res, 150));
    const filtered = data.filter(v => v.toLowerCase().startsWith(input.toLowerCase()));

    updatedSuggestions[idx] = data;
    activeSuggestion[idx] = filtered.length > 0 ? filtered[0] : null;

    return { updatedSuggestions, activeSuggestion };
};

export default fetchSuggestions;