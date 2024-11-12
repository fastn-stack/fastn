# Business Card Component

The **Business Card Component** is a simple and customizable way to display 
business card information within your `fastn` pacakge. With just a few lines of 
code, you can showcase your contact details, company information, even 
include a logo and others.

## Preview

Here's an example of how the Business Card Component might look when rendered:

### Front Side

![front.png](.github/assets/front.png)

### Back Side

![back.png](.github/assets/back.png)


## Getting Started

To use the Business Card Component in your `fastn` package, follow these steps:

1. **Add the Business Card Dependency**: Open your `FASTN.ftd` file and add 
   the following line to include the Business Card component:
   ```ftd
   -- fastn.dependency: fastn-community.github.io/pattern-business-card
   ```
2. **Use the Business Card Component**: In the file where you want to add 
   the business card (e.g., `index.ftd`), you can import the component and 
   use it like this:
    ```ftd
    -- import: fastn-community.github.io/pattern-business-card as b-card
    
    -- b-card.card: John Doe
    title: Software Developer
    company-name: John Doe Pvt. Ltd.
    logo: $assets.files.assets.ipsum-logo.svg
    contact-1: +91 12345 99999
    contact-2: +91 12345 88888
    email: john@johndoe.com
    website: www.johndoe.com
    address: 123, Block No. A-123, Times Square, Bangalore - 123456
    company-slogan: If you can type you can code
    ```
   
## Customization

Feel free to customize the business card by adding, removing, or modifying 
fields.

## Fields

- `title`: The job title or position.
- `company-name`: The name of your company or organization.
- `logo`: The path to your company logo file. Make sure to provide the correct 
  file path.
- `contact-1`: The primary contact number (optional).
- `contact-2`: An alternate contact number (optional).
- `email`: Your email address (optional).
- `website`: The URL of your website (optional).
- `address`: Your physical address or location (optional).
- `company-slogan`: A slogan or tagline for your company (optional).

Feel free to reach out if you have any questions or need further assistance. Happy coding!
