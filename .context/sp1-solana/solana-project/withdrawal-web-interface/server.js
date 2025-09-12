const express = require('express');
const cors = require('cors');
const multer = require('multer');
const path = require('path');
const fs = require('fs');

const app = express();
const PORT = process.env.PORT || 3000;

// Middleware
app.use(cors());
app.use(express.json());
app.use(express.static('.'));

// Configure multer for file uploads
const upload = multer({ dest: 'uploads/' });

// Routes
app.get('/', (req, res) => {
    res.sendFile(path.join(__dirname, 'index.html'));
});

// Generate proof endpoint
app.post('/api/generate-proof', async (req, res) => {
    try {
        const { user_address, pool_id, user_balance, withdrawal_amount, pool_liquidity } = req.body;
        
        // Validate input
        if (!user_address || !pool_id || !user_balance || !withdrawal_amount || !pool_liquidity) {
            return res.status(400).json({ error: 'Missing required fields' });
        }
        
        // In a real implementation, this would call the Rust proof generator
        // For now, we'll return a mock response
        const proofData = {
            proof: "0x" + "a".repeat(2000), // Mock proof
            public_inputs: "0x" + "b".repeat(100), // Mock public inputs
            vkey_hash: "0x" + "c".repeat(64), // Mock vkey hash
            mode: "compressed",
            user_address,
            pool_id: parseInt(pool_id),
            user_balance: parseInt(user_balance),
            withdrawal_amount: parseInt(withdrawal_amount),
            pool_liquidity: parseInt(pool_liquidity),
            timestamp: Date.now(),
            is_valid: true
        };
        
        res.json({
            success: true,
            data: proofData
        });
        
    } catch (error) {
        console.error('Error generating proof:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

// Verify proof endpoint
app.post('/api/verify-proof', upload.single('proofFile'), async (req, res) => {
    try {
        if (!req.file) {
            return res.status(400).json({ error: 'No proof file uploaded' });
        }
        
        const filePath = req.file.path;
        const fileContent = fs.readFileSync(filePath, 'utf8');
        const proofData = JSON.parse(fileContent);
        
        // In a real implementation, this would use the WASM verifier
        // For now, we'll return a mock verification result
        const isValid = Math.random() > 0.1; // 90% success rate for demo
        
        // Clean up uploaded file
        fs.unlinkSync(filePath);
        
        res.json({
            success: true,
            valid: isValid,
            data: proofData
        });
        
    } catch (error) {
        console.error('Error verifying proof:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

// Health check endpoint
app.get('/api/health', (req, res) => {
    res.json({ 
        status: 'healthy',
        timestamp: new Date().toISOString(),
        version: '1.0.0'
    });
});

// Start server
app.listen(PORT, () => {
    console.log(`🚀 Privacy-Preserving Pool System running on http://localhost:${PORT}`);
    console.log(`📊 Web Interface: http://localhost:${PORT}`);
    console.log(`🔍 API Health: http://localhost:${PORT}/api/health`);
});

