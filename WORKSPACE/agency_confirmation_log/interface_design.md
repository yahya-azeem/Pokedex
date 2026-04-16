# Agency Confirmation Interface Design

## Wireframe Description

The interface is designed to be a simple, two-step command-line interaction for confirming an agency.

**Step 1: Introduction and Action Prompt**
- The user is presented with a welcome message and an explanation of the process.
- They are then prompted to type a specific command ('CONFIRM') to proceed.

**Step 2: Confirmation and File Generation Notification**
- Upon successful input, a confirmation message is displayed.
- The system informs the user that the "physical proof" file has been generated and specifies its content and location.
- A final prompt to exit the interface.

## Mockups (Text-based UI)

### Screen 1: Welcome and Confirmation Prompt

```
======================================================
         Agency Confirmation Interface
======================================================

Welcome to the Agency Confirmation System.

This interface will guide you through confirming your agency
and generating the necessary proof file.

To proceed with the agency confirmation, please type 'CONFIRM'
and press Enter.

Enter your action:
>
```

### Screen 2: Confirmation and File Generation Output

```
======================================================
         Agency Confirmation Interface
======================================================

Confirmation Received!

Generating physical proof file...
File 'PHYSICAL_PROOF.txt' has been successfully created
in the WORKSPACE directory with the content 'AGENCY_CONFIRMED'.

Thank you for using the Agency Confirmation System.

Press Enter to exit.
```