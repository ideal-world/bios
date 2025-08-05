# Auth module

In the bios system, it is responsible for user authentication, authorization, encryption, decryption, and logging.

## Normal usage

The Auth module provides authentication and authorization functionality through the following methods:

### 1. Token-based Authentication

Users can authenticate using a token:
- Include the token in the request header using `Bios-Token`
- The system will validate the token and retrieve the associated account information
- If valid, the request will be authorized based on the account's permissions

### 2. AK/SK Authentication

For service-to-service authentication:
- Use Access Key (AK) and Secret Key (SK) for authentication
- Include the AK authorization in the `Bios-Authorization` header
- Request must include a timestamp in `Bios-Date` header (UTC time)
- System will validate the signature using the SK

### 3. Resource Protection

Resources can be protected with the following options:
- `need_login`: Requires user authentication
- `need_crypto_req`: Requires request encryption
- `need_crypto_resp`: Requires response encryption  
- `need_double_auth`: Requires secondary authentication

### 4. Authorization Rules

Access can be controlled based on:
- Specific accounts
- Roles
- Groups
- Applications
- Tenants
- Access Keys

### 5. Context Information

After successful authentication, the system will include context information in the response header `Tardis-Context` containing:
- Account ID
- Roles
- Groups
- Own paths
- Other relevant information

### 6. Resource Configuration

Resources are configured in a tree structure and stored in Redis:

#### Resource Structure
```json
{
    "uri": "protocol://path",
    "action": "http_method",
    "auth": {
        "accounts": "#account_id#",
        "roles": "#role_code#",
        "groups": "#group_code#",
        "apps": "#app_id#",
        "tenants": "#tenant_id#",
        "ak": "#ak#"
    },
    "need_crypto_req": false,
    "need_crypto_resp": false,
    "need_double_auth": false,
    "need_login": false
}
```

#### Storage Location
- Resource configurations are stored in Redis with key prefix `iam:res:info`
- Changes are tracked with key prefix `iam:res:changed:info:`
- System checks for changes every 30 seconds by default

#### Resource Matching
- Resources are matched by URI pattern and HTTP method
- Most specific match takes precedence
- URI matching supports wildcards and parameters

#### API Endpoints
- GET `/auth/apis` - Fetch server configuration and resource list
- PUT `/auth/` - Authenticate request against configured resources

## Third party usage

Third party use oauth2 flow to get access token.

### Oauth2 config source

#### 1. sign up application
sign up application in the BIOS, get client id and client secret.

store the client id and client secret in to rbum_cert,and cache key in redis.
#### 2. auth get oauth2 config
auth get oauth2 config from cache key in redis.

### Third party integration oauth2

#### 1. 

