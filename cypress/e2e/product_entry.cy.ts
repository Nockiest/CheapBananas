3// <reference types="cypress" />

describe('Product Entry Form', () => {
  beforeEach(() => {
    cy.visit('http://localhost:3000');
  });

  it('submits a product entry with unit conversion (g → kg)', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana 20 10 g tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('sent to backend', { timeout: 5000 }).should('be.visible');
  });

  it('submits a product entry with unit conversion (hl → l)', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana 10 0.5 hl tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('sent to backend', { timeout: 5000 }).should('be.visible');
  });

  it('submits a product entry with price per piece normalization (ks)', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana 30 30 ks tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('sent to backend', { timeout: 5000 }).should('be.visible');
  });

  it('submits a product with tags', () => {
    cy.contains('Product').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana some-notes tag1,tag2');
    cy.get('button').contains('Send to backend').click();
    cy.contains('sent to backend', { timeout: 5000 }).should('be.visible');
  });

  it('shows error if required fields are missing', () => {
    cy.contains('Product Entry').click();
    cy.get('input[placeholder^="Type:"]').clear().type('banana');
    cy.get('button').contains('Send to backend').should('be.disabled');
    cy.contains('Please fill in all required fields').should('be.visible');
  });
});
