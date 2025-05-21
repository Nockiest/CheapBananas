// <reference types="cypress" />

describe('Product Entry Form', () => {

    beforeEach(() => {
  // Ensure the product does not exist, then create it
  cy.request({
    method: 'DELETE',
    url: 'http://localhost:4000/products/00000000-0000-0000-0000-000000000001',
    failOnStatusCode: false // ignore 404 errors
  });
  cy.request('POST', 'http://localhost:4000/products', {
    id: '00000000-0000-0000-0000-000000000001',
    name: 'banana',
    notes: '',
    tags: []
  });
  // Add shops needed for tests
  ['tesco', 'lidl', 'albert', 'billa'].forEach(shopName => {
    cy.request('POST', 'http://localhost:4000/shops', {
      name: shopName,
      notes: ''
    });
  });
  cy.visit('http://localhost:3000');
});
  it('submits a product entry with unit conversion (g → kg)', () => {
    cy.contains('Product Entry').click();
    // Fill all fields, use _ for optional ones, date is now last
    cy.get('input[placeholder^="Type:"]').clear().type('banana 20 10 g tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Product Entry sent to backend!', { timeout: 5000 }).should('be.visible');
  });

  it('submits a product entry with unit conversion (hl → l)', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana 10 0.5 hl tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Product Entry sent to backend!', { timeout: 5000 }).should('be.visible');
  });

  it('submits a product entry with price per piece normalization (ks)', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana 30 30 ks tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Product Entry sent to backend!', { timeout: 5000 }).should('be.visible');
  });

  it('shows error if required fields are missing', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana');
    cy.get('button').contains('Send to backend').should('be.disabled');
    cy.contains('Please fill in all required fields').should('be.visible');
  });

  it('submits a product entry with empty optional fields', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana 10 1 kg tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Product Entry sent to backend!', { timeout: 5000 }).should('be.visible');
  });

  it('submits a product entry with extra spaces between fields', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana   10   1   kg   tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Product Entry sent to backend!', { timeout: 5000 }).should('be.visible');
  });

  it('shows error for non-existent product', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('notarealproduct 10 1 kg tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('error', { matchCase: false, timeout: 5000 }).should('be.visible');
  });

  it('shows error for non-existent shop', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana 10 1 kg notarealshop');
    cy.get('button').contains('Send to backend').click();
    cy.contains('error', { matchCase: false, timeout: 5000 }).should('be.visible');
  });

  it('shows error for invalid price (negative)', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana -5 1 kg tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('error', { matchCase: false, timeout: 5000 }).should('be.visible');
  });

  it('shows error for invalid product volume (negative)', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana 10 -1 kg tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('error', { matchCase: false, timeout: 5000 }).should('be.visible');
  });

  it('shows error for invalid unit', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana 10 1 xyz tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('error', { matchCase: false, timeout: 5000 }).should('be.visible');
  });

  it('submits a product entry with tags and notes', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana 10 1 kg tesco tag1,tag2 some-notes');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Product Entry sent to backend!', { timeout: 5000 }).should('be.visible');
  });

  it('submits a product entry with a date', () => {
    cy.contains('Product Entry').click();
    const today = new Date().toISOString().split('T')[0];
    cy.get('input[placeholder^="Type:"]').clear().type(`banana 10 1 kg tesco _ _ ${today}`);
    cy.get('button').contains('Send to backend').click();
    cy.contains('Product Entry sent to backend!', { timeout: 5000 }).should('be.visible');
  });

  it('shows error for missing all required fields', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('');
    cy.get('button').contains('Send to backend').should('be.disabled');
    cy.contains('Please fill in all required fields').should('be.visible');
  });
});
