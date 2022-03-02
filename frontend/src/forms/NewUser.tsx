import { User } from "../types";
import * as React from 'react';

type NewUser = {
    username: string,
};

type Props = {
    submissionHandler: React.FormEventHandler<HTMLFormElement>,
};

class NewUserForm extends React.Component<Props, NewUser> {
    state: NewUser = {username: ""};

    handleChange: React.ChangeEventHandler<HTMLInputElement> = (event) => {
        this.setState({ username: event.currentTarget.value});
    }

    render() {
        return (
            <div className="newUserForm">
                <form onSubmit={this.props.submissionHandler}>
                    <label htmlFor="username">Username</label>
                    <input type="text" name="username" onChange={this.handleChange} value={this.state.username} />
                    <input type="submit" />
                </form>
            </div>
        )
    }
}

export default NewUserForm;
