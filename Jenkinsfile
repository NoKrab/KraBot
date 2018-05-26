pipeline {
    agent {
        label 'Ubuntu'
    }

    stages {
        stage('Build') {
            steps {
                sh "cargo build"
            }
        }
    }
}