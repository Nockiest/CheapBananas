// <reference types="cypress" />

describe('Product Form', () => {
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
    cy.visit('http://localhost:3000');
  });

  it('submits a product with tags', () => {
    cy.contains('Product').click();
    // For Product: Name Notes Tags (only 3 fields)
    cy.get('input[placeholder^="Type:"]').clear().type('banana notes tag1,tag2');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Product sent to backend!', { timeout: 5000 }).should('be.visible');
  });

  it('shows error if required fields are missing', () => {
    cy.contains('Product').click();
    cy.get('input[placeholder^="Type:"]').clear().type('');
    cy.get('button').contains('Send to backend').should('be.disabled');
    cy.contains('Please fill in all required fields').should('be.visible');
  });

  it('submits a product with only name', () => {
    cy.contains('Product').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Product sent to backend!', { timeout: 5000 }).should('be.visible');
  });

  it('submits a product with name and notes', () => {
    cy.contains('Product').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana some-notes');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Product sent to backend!', { timeout: 5000 }).should('be.visible');
  });

  it('submits a product with name and tags', () => {
    cy.contains('Product').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana _ tag3,tag4');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Product sent to backend!', { timeout: 5000 }).should('be.visible');
  });
});
