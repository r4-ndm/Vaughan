// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../lib/openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

/// @title CustomToken
/// @notice A customizable ERC20 token with parameters for name, symbol, and supply
contract CustomToken is ERC20 {
    uint8 private immutable _customDecimals;
    
    /// @notice Creates a new token with custom parameters
    /// @param name_ The name of the token (e.g., "My Token")
    /// @param symbol_ The symbol of the token (e.g., "MTK")
    /// @param initialSupply_ The initial supply (will be multiplied by 10^decimals_)
    /// @param decimals_ The number of decimals (typically 18)
    constructor(
        string memory name_,
        string memory symbol_,
        uint256 initialSupply_,
        uint8 decimals_
    ) ERC20(name_, symbol_) {
        _customDecimals = decimals_;
        // Mint initial supply to the deployer
        _mint(msg.sender, initialSupply_ * 10**decimals_);
    }
    
    /// @notice Returns the number of decimals
    function decimals() public view virtual override returns (uint8) {
        return _customDecimals;
    }
}
