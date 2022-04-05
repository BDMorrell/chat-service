// import * as React from "react";
import NewUserForm from "./forms/NewUser";

const App = () => (
    <>
        <p>Hello!</p>
        <NewUserForm submissionHandler={(e) => {e.preventDefault()}}/>
    </>
);

export default App;
