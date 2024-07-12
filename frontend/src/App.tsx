import { useAuth0 } from "@auth0/auth0-react";

function App() {
  const auth0 = useAuth0();
  const { user, isAuthenticated, isLoading, loginWithRedirect, logout } = auth0;

  if (isLoading) {
    return <div>Loading ...</div>;
  }

  if (isAuthenticated && user) {
    return (
      <div>
        <img src={user.picture} alt={user.name} />
        <h2>{user.name}</h2>
        <p>{user.email}</p>
        <button onClick={() => logout({ logoutParams: { returnTo: window.location.origin } })}>
          Log Out
        </button>
        <button
          onClick={async () => {
            const accessToken = await auth0.getAccessTokenWithPopup({ authorizationParams: { audience: import.meta.env.VITE_AUTH0_AUDIENCE } });

            fetch("http://localhost:3000/", {
              headers: {
                "AUTHORIZATION": `Bearer ${accessToken}`,
              }
            })
              .then(response => response.text())
              .then(console.log);
          }}
        >
          Call API
        </button>
      </div>
    )
  }

  return (
    <div>
      <button onClick={() => loginWithRedirect()}>Log In</button>
    </div>
  )
}

export default App
