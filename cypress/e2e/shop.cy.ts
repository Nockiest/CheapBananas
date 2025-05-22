// <reference types="cypress" />

describe('Shop Form', () => {
  beforeEach(() => {
    // Ensure the shop does not exist, then create it
    ['tesco', 'lidl', 'albert', 'billa'].forEach(shopName => {
      try {
        cy.request({
          method: 'DELETE',
          url: `http://localhost:4000/shops/${shopName}`,
          failOnStatusCode: false // ignore 404 errors
        });
      } catch (error) {
        cy.log(`Failed to connect to backend for shop: ${shopName}`);
      }
    });
    cy.visit('http://localhost:3000');
  });

  it('submits a shop with name only', () => {
    cy.contains('Shop').click();
    cy.get('input[placeholder^="Type:"]').clear().type('tesco');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Shop sent to backend!', { timeout: 5000 }).should('be.visible');
  });

  it('submits a shop with name and notes', () => {
    cy.contains('Shop').click();
    cy.get('input[placeholder^="Type:"]').clear().type('lidl some-notes');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Shop sent to backend!', { timeout: 5000 }).should('be.visible');
  });

  it('shows error if required fields are missing', () => {
    cy.contains('Shop').click();
    cy.get('input[placeholder^="Type:"]').clear().type('');
    cy.get('button').contains('Send to backend').should('be.disabled');
    cy.contains('Please fill in all required fields').should('be.visible');
  });

  it('prevents duplicate shop creation', () => {
    cy.contains('Shop').click();
    cy.get('input[placeholder^="Type:"]').clear().type('albert');
    cy.get('button').contains('Send to backend').click();
    cy.contains('Shop sent to backend!', { timeout: 5000 }).should('be.visible');
    // Try to add again
    cy.get('input[placeholder^="Type:"]').clear().type('albert');
    cy.get('button').contains('Send to backend').click();
    cy.contains('error', { matchCase: false, timeout: 5000 }).should('be.visible');
  });
});
