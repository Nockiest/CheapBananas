export function replaceUnderscoresWithNull(obj: any): any {
  if (typeof obj === 'string') {
    // Replace strings with only underscores with null
    return obj === '_'.repeat(obj.length) ? null : obj.replace(/_/g, '');
  } else if (Array.isArray(obj)) {
    // Recursively process each element in the array
    return obj.map(replaceUnderscoresWithNull);
  } else if (typeof obj === 'object' && obj !== null) {
    // Recursively process each key-value pair in the object
    return Object.fromEntries(
      Object.entries(obj).map(([key, value]) => [key, replaceUnderscoresWithNull(value)])
    );
  }
  // Return the value as-is if it's not a string, array, or object
  return obj;
}

// Example usage:
const input = {
  name: "example_name",
  details: {
    description: "some_description",
    tags: ["tag_one", "tag_two"],
  },
  items: [
    { id: "item_one", value: "value_one" },
    { id: "item_two", value: "value_two" },
    { id: "____", value: "_" },
  ],
};
