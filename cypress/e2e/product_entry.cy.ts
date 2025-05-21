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
});
