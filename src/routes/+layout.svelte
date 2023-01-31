<script>
    import "bootstrap/dist/css/bootstrap.min.css";
    import { Nav, NavItem, NavLink, Navbar, NavbarBrand } from "sveltestrap";
</script>

<Navbar color="dark" dark class="mb-3" expand="md">
    <NavbarBrand href="/">GHD</NavbarBrand>
    <Nav class="ms-auto" navbar>
        <NavItem>
            <NavLink href="/">Home</NavLink>
        </NavItem>
        <NavItem>
            <NavLink href="/settings">Settings</NavLink>
        </NavItem>
    </Nav>
</Navbar>

<div class="container-fluid mx-2">
    <slot></slot>
</div>